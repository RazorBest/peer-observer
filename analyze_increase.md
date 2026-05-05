# Analyzing the increase function of Prometheus


[The documentation](https://prometheus.io/docs/prometheus/latest/querying/functions/#increase) for the function states:
> increase(v range-vector) calculates the increase in the time series in the range vector. Breaks in monotonicity (such as counter resets due to target restarts) are automatically adjusted for. The increase is extrapolated to cover the full time range as specified in the range vector selector, so that it is possible to get a non-integer result even if a counter increases only by integer increments.

> increase should only be used with counters (for both floats and histograms). It is syntactic sugar for rate(v) multiplied by the number of seconds under the specified time range window, and should be used primarily for human readability. Use rate in recording rules so that increases are tracked consistently on a per-second basis.

When I first read that, I found it lacking in detail. I needed a mathematical definition of `increase`. So I built one myself, starting from the basics.


## Range vector

One of the types commonly used for time series is a range vector:
```
v[range] = (t1, v1), (t2, v2), (t3, v3), ..., (tn, vn)
```
Which is a vector a 2-paired elements, where each element has a timestamp and a value. The timestamps are increasing. The range acts like a selector. It can be represented by an interval defined by `T_start` and `T_end`, and selects all the values in the vector whose timestamps are in that interval: `T_start <= ti <= T_end`. Prometheus encourages you to use a single value for ranges, and `T_end` is assumed to be the current time. For example `v[5h]` is selecting an interval defined by `T_end = now()` and `T_start = T_end - 5h`.

## Monotonicity

Let's first clear one of the easier issues when it comes to time series: **counter resets**.

> Breaks in monotonicity (such as counter resets due to target restarts) are automatically adjusted for

How is the adjustment done, exactly?

Meaning that at some point, you would have $v_{i} > v_{i+1}$. If you just select a range vector that contains this, the corresponding values will be plotted - nothing unexpected. However, when you need to apply some functions to these range vectors, Prometheus will make the following assumption: if a break in monotonicity happens, it means that the counter was reset to 0. In its calculations, it will add an offset every time this happens, to turn the series into a monotonic one.

This means that the new value will be $v_{i+1}' = v_{i} + v_{i+1}$. For example, if a drop to 0 happens (i.e. $v_{i+1}$ becomes 0), that sample will actually take the value $v_{i+1}' = v_{i}$, as if no change happened.

The following property is reinforced by the second paragraph of the documentation of the `increase` function:
> increase should only be used with counters (for both floats and histograms)

Here's a visual example. The following is a counter with resets:
![Plot of counter that has 2 resets](img/counter_initial.png "A counter with resets")

And here it is how Prometheus will interpret it when given to the `increase` function:
![Plot of the same counter without resets](img/counter_no_resets.png "The same counter without resets")


## Formula

```
range := (T_start, T_end)
factor = (T_start, T_end) / (tn - t1)
raw_delta = tn - t1
increase(v[range]) = raw_delta * factor
```
