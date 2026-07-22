# ecodrive

This project is built on the following sources:

[1] R. Franke, P. Terwiesch, M. Meyer, C. Klose, K.H. Ketteler, "Method for optimizing energy in the manner in which a vehicle or train is driven using kinetic energy."
[US6668217B1](https://patentimages.storage.googleapis.com/f2/ae/b2/485577511da65e/US6668217.pdf)

[2] W. Liljenström, M. Rothhämel, "Optimizing energy use in electric vehicles: A comparative study of optimization methods." Rev2025 –2nd Resource Efficient Vehicles Conference. 2026. https://doi.org/10.25364/978-3-903374-48-5_28

## Description of the underlying physical model

The model works with mass-normalized values, thus the mass never explicitly appears in the calculations.

Specific kinetic energy:
$$e_{\mathrm{kin}} = \frac{e_{\mathrm{kin}}}{m\cdot \rho_{\mathrm{rot}}} = \frac{1}{2}v^2$$
$\rho_{\mathrm{rot}}$ is a factor to account for the rotating masses

The dynamics are modeled by the following differential equation: 

$$\frac{\mathrm{d}e_{\mathrm{kin}}}{\mathrm{d}s} = A - B \cdot \sqrt{e_{\mathrm{kin}}} - C \cdot e_{\mathrm{kin}}$$

$$\frac{\mathrm{d}t}{\mathrm{d}s} = \frac{1}{\sqrt{2}} \cdot \sqrt{\frac{1}{e_{\mathrm{kin}}}}$$

The solution on one section with constant $A$ and constant $C$ is given by:

$$e_{\mathrm{kin}}(s) = \frac{A}{C} + \left(e_{\mathrm{kin}}(0) - \frac{A}{C} \right) \cdot \exp(-C \cdot s)\qquad \mathrm{(1)}$$

$A = \rho_{\mathrm{rot}} \cdot \tau - g \cdot \tan\theta - C_r \cdot g$

$B = 0$

$C = \rho_{\mathrm{air}} \cdot A_{\mathrm{front}} \cdot c_{\mathrm{d}} \cdot \frac{1}{m}$

Each of these forces is their component in horizontal direction (division by $\cos\theta$). $\tau$ is always used in direct opposition to $F_{\mathrm{roll}}$, $F_{\mathrm{incl}}$ and $F_{\mathrm{air}}$.

$F_{\mathrm{air}} = \frac{1}{2} \cdot \rho_{\mathrm{air}} \cdot A \cdot c_{\mathrm{d}} \cdot v^2/\cos{\theta} = \rho_{\mathrm{air}} \cdot A \cdot c_{\mathrm{d}} \cdot e_{\mathrm{kin}} / \cos{\theta}$

$F_{\mathrm{roll}} = C_r \cdot m \cdot g$

$F_{\mathrm{incl}} = m \cdot g \cdot \tan\theta$

$F_{\mathrm{\tau}} = m \cdot \rho_{\mathrm{rot}} \cdot \tau$

$\tan\theta = slope$

In each section of length $s$, the parameters $A$ and $C$ are constant and $e_{\mathrm{kin}}(s)$ can be calculated from $e_{\mathrm{kin}}(0)$ using the solution (1) to the differential equation model.

## Dynamic Programming (DP)

There are two modes of optimization:
- optimizing used energy, given a time budget
- optimizing used time, given an energy budget

The following section explains the minimization of used energy, with the [minimization of used time in braces].

Starting from one individual state $(v, t)$ [or: $(v, e)$], all possible next states $(v_{\mathrm{next}}, t_{\mathrm{next}})$ [or: $(v_{\mathrm{next}}, e_{\mathrm{next}})$] are tested and the associated value for the used energy is saved. If a path reaches the same state with a lower used energy, it replaces the previous path to that state. Thus, after finishing a step, each saved path to a certain state is optimal.

Not all paths are pursued until the end. Instead, forbidden paths are discarded already during the optimization. A path can be forbidden due to:
- exceeding the maximum time [the available energy]
- violating the speed limit
- being impossible to reach within the limits for the momentum

Thus, no viable path is discarded prematurely and the resulting optimal path is the same as if no paths would have been discarded at all.

Each reached state is associated with the used energy [used time] and its parent state. This way, a path can be reconstructed by starting from the last state and backtracking through the parent states.

The result of the optimization is an optimal path, consisting of $v$ and $t$ for each section boundary. This path uses the least amount of energy [time] within the given time [energy] bounds. This minimal amount is returned as well.

## Tips

Use `cargo run --release` for substantially increased performance.

Use `optim_time()` with 100% `soc` and manually cap the speed limits to a maximum velocity to get a "naive" schedule for comparison, where maximum acceleration and deceleration are used, while the total maximum speed is limited.

## Possible further steps

Evaluate the effect of a finer splitting of the sections on the result. One approach could be to introduce an additional section boundary shortly after the beginning of a section with higher speed limit. This would allow for a short phase of high acceleration and then holding a constant speed.

Using sparse matrices could yield a performance (but certainly a memory) benefit, as most entries of possible states are never used.

Implementing an automatic download of routes from a public map API.
