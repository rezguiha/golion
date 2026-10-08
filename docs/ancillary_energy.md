# Ancillary energy constraint: conservative approximation

## Constraint

For every activation start $t$ and every offset $0 \le j < \max_m W_m$:

$$
\mathrm{SoC}_{t+j} - \frac{1}{\eta_d}\, E^{\uparrow}_{t,j} \;\ge\; \mathrm{SoC}^{\min}_{t+j}
$$

$$
\mathrm{SoC}_{t+j} + \frac{1}{\eta_d}\, E^{\downarrow}_{t,j} \;\le\; \mathrm{SoC}^{\max}_{t+j}
$$

with the activation energy drawn at the grid connection since the start:

$$
E^{\uparrow}_{t,j} = \sum_{k=t}^{t+j} \; \sum_{m \,:\, k-t < W_m} R^{\uparrow}_{m,k}\, \Delta t
\qquad
E^{\downarrow}_{t,j} = \sum_{k=t}^{t+j} \; \sum_{m \,:\, k-t < W_m} R^{\downarrow}_{m,k}\, \Delta t
$$

- $\mathrm{SoC}$ is the planned state of charge, which already includes the planned
  wholesale dispatch.
- $R^{\uparrow}_{m,k}$ and $R^{\downarrow}_{m,k}$ are the upward and downward reserve
  powers of reserve $m$ held by the asset at step $k$.
- $W_m$ is the activation window of reserve $m$, in steps.
- $\eta_c$ and $\eta_d$ are the charge and discharge efficiencies.

## Why it is conservative

The exact state of charge change per kWh activated at the grid depends on the planned
state of the battery during the step:

| Activation | Delivered by | State of charge change per kWh |
|---|---|---|
| upward | extra discharge | $-1/\eta_d$ |
| upward | cut of a planned charge | $-\eta_c$ |
| downward | extra charge | $+\eta_c$ |
| downward | cut of a planned discharge | $+1/\eta_d$ |

Since

$$
\eta_c \;\le\; 1 \;\le\; \frac{1}{\eta_d}
$$

using $1/\eta_d$ in both directions always takes the largest change.

- **Never underestimates:** the energy the battery must hold is always covered.
- **Exact** for upward activation while idle or discharging, and for downward activation
  cutting a planned discharge.
- **Overestimates** in the other cases, by at most a factor $\frac{1}{\eta_c\,\eta_d}$,
  the inverse of the round-trip efficiency.
- **No binaries:** an exact treatment would need one binary per step and per activation
  start.
