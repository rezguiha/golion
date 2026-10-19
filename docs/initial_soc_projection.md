# Initial state of charge projection

The plan starts at the first step boundary after the run time. During this `gap`, the battery
follows the step in progress: `dispatch` (positive for charge), `reserve_up` and
`reserve_down`.

```text
delta_soc    = dispatch * gap * eta_c    if dispatch > 0
               dispatch * gap / eta_d    otherwise

soc_start    = clamp(soc_measured + delta_soc, soc_min, soc_max)
up_pending   = min(reserve_up   * gap, (soc_start - soc_min) * eta_d)
down_pending = min(reserve_down * gap, (soc_max - soc_start) * eta_d)
```

- `soc_start` is the initial state of charge of the plan. It applies the same losses as the
  plan, and it is clamped because the battery stops at its limits.
- `up_pending` and `down_pending` are the worst-case activations during the gap. The room
  left in the battery, reduced by `eta_d`, caps how much activation it can handle. This is
  the same conservative factor as in the
  [ancillary energy reservation](ancillary_energy.md). They shift the state of charge for
  the whole horizon, so every activation scenario starts from them:

```text
soc[t] - (up_pending   + upward_activation)   / eta_d >= soc_min[t]
soc[t] + (down_pending + downward_activation) / eta_d <= soc_max[t]
```
