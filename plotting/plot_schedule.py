import sys
import csv
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.colors
from matplotlib.cm import ScalarMappable

GLOBAL_MOM_MAX = 3 # [m/s^2]
MAX_SLOPE = 30 # [%]
g = 9.81 # m/s^2

# TODO: use C_r of loaded vehicle instead
# TODO: make row argument of vehicle optional, so that by default the first vehicle from the given file is used
# TODO: include function to load schedule
# TODO: write function to do the data preparation and call to plot_schedule (or do the data preparation and call separately in main)

def load_vehicle(path, row=0):
    vehicles = np.loadtxt(path, skiprows=1, delimiter=',')

    with open(path, newline='') as file:
        reader = csv.DictReader(file)
        for i, vehicle in enumerate(reader):
            if i == row:
                print(i, vehicle)
                return vehicle


def f_route_res(slopes):
    '''Route resistance s.t. A = mom * rho_rot - route_res
    slopes [%]'''
    return g * ((slopes / 100) + C_r)

def retrieve_A(s, ekin_0, ekin_s, C=C_default):
    '''Calculate A parameter necessary to get from `ekin_0` to `ekin_s` on length `s`.
    Used in backtracking to retrieve moment schedule of a path.'''
    A = C * (ekin_s - ekin_0 * np.exp(-C * s)) / (1 - np.exp(-C * s))
    return A

def e_kin(s, A, C, e_kin0):
    '''Kinetic energy at point s'''
    return A/C + (e_kin0 - A/C) * np.exp(-C*s)

def ekin_to_v(ekin):
    '''Inverse of v_to_ekin
    ekin [m^2/s^2]
    returns v [m/s]'''
    if np.isclose(ekin, 0):
        return 0
    if ekin < 0:
        return -1
    return np.sqrt(2 * ekin)

ekin_to_v_vectorized = np.vectorize(ekin_to_v)

def plot_schedule(lengths, As, moments, v_maxs=None, route_res=None, num_x=10000, ekin_0=0, cmap_moment_name="RdBu_r", cmap_route_res_name="PuOr_r", only_opt_path=False):
    """Plot the velocity and moment of a given path over the route."""
    total_dist = lengths.sum()
    x_tot = np.array([0])
    y_tot = np.array([ekin_to_v(ekin_0) * 3.6])
    nums = []
    previous_dists = np.concatenate(([0], np.cumsum(lengths)))
    for s, prev_dist, A in zip(lengths, previous_dists, As):
        # get appropriate number of x values for given section
        num = int(num_x * (s / total_dist))
        nums = nums + [num]

        if num <= 1: # then there would be 0 new x-values
            nums[-1] = 0
            continue
        
        x = np.linspace(0, s, num+1)[1:]
        
        # calculate every e_kin for current x-section
        ekins = e_kin(x, A, C_default, ekin_0)

        # set new starting ekin for next section
        ekin_0 = ekins[-1]

        # convert to v for plotting
        y = ekin_to_v_vectorized(ekins) * 3.6 # y in [km/h]

        # shift x to cover current section
        x += prev_dist

        x_tot = np.concatenate((x_tot, x))
        y_tot = np.concatenate((y_tot, y))
    
    if not only_opt_path:
        fig, ax = plt.subplots(layout="constrained", figsize=(9, 5))
        cmap_moment = plt.get_cmap(cmap_moment_name)
        cmap_route_res = plt.get_cmap(cmap_route_res_name)

        # plot maximum velocities
        if v_maxs is not None:
            v_maxs = v_maxs * 3.6 # now in [km/h]

            plt.step(previous_dists, np.concatenate((v_maxs, [v_maxs[-1]])), "k:", where="post", label="speed limit")

            # find maximum displayed y-value
            y_max = max(np.max(y_tot), np.max(v_maxs))

        else:
            # find maximum displayed y-value
            y_max = np.max(y_tot)

        base_level = np.min(y_tot) - 0.02 * (np.max(y_tot) - np.min(y_tot))
        top_level  = y_max + 0.02 * (np.max(y_tot) - np.min(y_tot))

        max_route_res = f_route_res(MAX_SLOPE)
        min_route_res = f_route_res(-MAX_SLOPE)

        start_idx = 0
        for i, num in enumerate(nums):
            # show applied moment colorcoded below curve
            plt.fill_between(x_tot[start_idx:start_idx+num+1], y_tot[start_idx:start_idx+num+1], base_level, color=cmap_moment(moments[i] / (2 * GLOBAL_MOM_MAX) + 0.5))

            if route_res is not None:
                # show route resistance colorcoded above curve
                plt.fill_between(x_tot[start_idx:start_idx+num+1], top_level, y_tot[start_idx:start_idx+num+1], color=cmap_route_res((route_res[i] - min_route_res) / (max_route_res - min_route_res)))
            
            start_idx += num

        fig.colorbar(ScalarMappable(norm=matplotlib.colors.Normalize(-GLOBAL_MOM_MAX, GLOBAL_MOM_MAX), cmap=cmap_moment_name), ax=ax, pad=0.02, label="applied moment [m/$\\mathrm{s^2}$]")
        fig.colorbar(ScalarMappable(norm=matplotlib.colors.Normalize(min_route_res, max_route_res), cmap=cmap_route_res_name), ax=ax, pad=0.03, label="route resistance [m/$\\mathrm{s^2}$]")

    # plot optimal path
    alpha_opt = 0.2 if only_opt_path else 1.0
    label = None if only_opt_path else "optimal schedule"
    plt.plot(x_tot, y_tot, "black", label=label, alpha=alpha_opt)

    plt.legend(loc="lower center")
    plt.xlabel("distance [m]")
    plt.ylabel("velocity [km/h]")

if __name__ == "__main__":
    print("executing plot_schedule.py")

    load_vehicle("../vehicle1.csv")
    if len(sys.argv) <= 1:
        print("not enough arguments! Please provide a file name and optionally a destination")

    elif len(sys.argv) == 2:
        plot_schedule(sys.argv[1], None)

    elif len(sys.argv) == 3:
        plot_schedule(sys.argv[1], sys.argv[2])

    else:
        print("too many arguments! Please only provide a file name and optionally a destination")
    