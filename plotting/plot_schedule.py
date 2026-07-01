import sys
import numpy as np
import os.path
import matplotlib.pyplot as plt
import matplotlib.colors
from matplotlib.cm import ScalarMappable

# constants, should agree with the ones defined in ecodrive::constants
GLOBAL_MOM_MAX = 3 # [m/s^2]
MAX_SLOPE = 30 # [%]
RHO_AIR = 1.2 # [kg/m^3]
g = 9.81 # m/s^2


def load_route(route_name, repeat=1):
    """Load route from .csv file, converting km/h to m/s"""
    # add .csv file ending if not provided
    if os.path.splitext(route_name)[1] == "":
        route_name = route_name + ".csv"

    route = np.genfromtxt(route_name, names=True, delimiter=',')

    lengths_m = np.repeat(route["length_m"], repeat) / repeat # in m, divide by repeat so total length stays the same
    slopes_pct = np.repeat(route["slope_pct"], repeat) # in %
    max_speeds_mps = np.repeat(route["max_speed_kmh"] / 3.6, repeat) # convert to m/s

    return {"lengths_m": lengths_m, "slopes_pct": slopes_pct, "max_speeds_mps": max_speeds_mps}


def load_vehicle(path, row=0):
    """Load vehicle from .csv file, optionally choosing a row. Defaults to the first row"""
    # add .csv file ending if not provided
    if os.path.splitext(path)[1] == "":
        path = path + ".csv"
    
    with open(path, newline='') as file:
        vehicles = np.genfromtxt(path, names=True, delimiter=',')
        return vehicles[row]


def load_schedule(path):
    """Load schedule from .csv file, converting km/h to m/s"""
    # add .csv file ending if not provided
    if os.path.splitext(path)[1] == "":
        path = path + ".csv"

    schedule = np.genfromtxt(path, names=True, delimiter=',')
    
    speed_mps = schedule["speed_kmh"] / 3.6

    return {"time_s": schedule["time_s"], "speed_mps": speed_mps}


def calculate_C(vehicle):
    return RHO_AIR * vehicle["c_w"] * vehicle["frontal_area_sqm"] / vehicle["mass_kg"]


def f_route_res(slopes, C_r):
    """Route resistance s.t. A = mom * rho_rot - route_res
    slopes [%]"""
    return g * ((slopes / 100) + C_r)


def retrieve_A(s, ekin_0, ekin_s, C):
    """Calculate A parameter necessary to get from `ekin_0` to `ekin_s` on length `s`.
    Used in backtracking to retrieve moment schedule of a path."""
    A = C * (ekin_s - ekin_0 * np.exp(-C * s)) / (1 - np.exp(-C * s))
    return A


def A_to_moment(A, route_res, rho_rot):
    """Retrieve actual moment from A parameter."""
    moment = (A + route_res) / rho_rot
    return moment


def e_kin(s, A, C, e_kin0):
    """Kinetic energy at point s"""
    return A/C + (e_kin0 - A/C) * np.exp(-C*s)


def v_to_ekin(v):
    '''v [m/s]'''
    return 0.5 * np.square(v)


def ekin_to_v(ekin):
    """Inverse of v_to_ekin
    ekin [m^2/s^2]
    returns v [m/s]"""
    if np.isclose(ekin, 0):
        return 0
    if ekin < 0:
        return -1
    return np.sqrt(2 * ekin)

ekin_to_v_vectorized = np.vectorize(ekin_to_v)


def create_plot(save_to, lengths, As, moments, C_param, C_r, v_maxs=None, route_res=None, num_x=10000, ekin_0=0, cmap_moment_name="RdBu_r", cmap_route_res_name="PuOr_r", only_opt_path=False):
    """Plot the velocity and moment of a given path over the route.
    y-axis: velocity (solid) and speed limit (dotted)
    colors above the curve: route resistance (slope + rolling resistance)
    colors below the curve: applied moment"""
    total_dist = lengths.sum()
    x_tot = np.array([0])
    y_tot = np.array([ekin_to_v(ekin_0) * 3.6]) # [km/h]
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
        ekins = e_kin(x, A, C_param, ekin_0)

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

        max_route_res = f_route_res(MAX_SLOPE, C_r)
        min_route_res = f_route_res(-MAX_SLOPE, C_r)

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

    plt.savefig(save_to)


def plot_schedule(schedule_path, route_path, vehicle_path, vehicle_num=0, save_to=None):
    """Load data, prepare arguments and call create_plot()"""
    # load data
    vehicle = load_vehicle(vehicle_path)
    route = load_route(route_path)
    lengths, slopes, max_speeds = route["lengths_m"], route["slopes_pct"], route["max_speeds_mps"]
    schedule = load_schedule(schedule_path)

    # prepare arguments for create_plot()
    e_kins = (v_to_ekin(schedule["speed_mps"]))
    route_res = f_route_res(slopes, vehicle["roll_res_coeff"])

    C_param = calculate_C(vehicle)
    C_r = vehicle["roll_res_coeff"]

    A_retrieved = retrieve_A(lengths, e_kins[:-1], e_kins[1:], C_param)
    moments = A_to_moment(A_retrieved, route_res, vehicle["rho_rot"])

    # retrieve schedule name from path
    tail = os.path.split(schedule_path)[1]
    schedule_name = os.path.splitext(tail)[0]

    # set default name if no destination given
    if save_to == None:
        dirname = os.path.dirname(__file__)
        filename = os.path.join(dirname, f'figures/{schedule_name}.png')
        save_to = filename

    # add .png if destination has no file extension
    elif os.path.splitext(save_to)[1] == "":
        save_to = save_to + ".png"

    # use given destination as-is
    else:
        pass

    # create and save plot
    create_plot(save_to, lengths, A_retrieved, moments, C_param, C_r, max_speeds, ekin_0=e_kins[0])


if __name__ == "__main__":
    print("executing plot_schedule.py")

    # either change these to the desired paths or supply them as command line arguments when calling plot_route.py
    default_schedule = "results/route3_res8_result.csv"
    default_route = "routes/route3_res8.csv"
    default_vehicle = "../vehicles.csv"
    default_vehicle_row = 0
    default_destination = "figures/route3_res8_result.png"

    # without arguments, use default values
    if len(sys.argv) <= 1:
        print("using default arguments")
        plot_schedule(default_schedule, default_route, default_vehicle, default_vehicle_row, default_destination)

    elif len(sys.argv) <= 3:
        print("not enough arguments! Please provide schedule, route, vehicle and optionally vehicle row and destination")

    # schedule, route, vehicle
    elif len(sys.argv) == 4:
        plot_schedule(*sys.argv[1:])

    # schedule, route, vehicle, vehicle row
    # OR schedule, route, vehicle, destination
    elif len(sys.argv) == 5:
        try:
            vehicle_row = int(sys.argv[4])
            plot_schedule(sys.argv[1], sys.argv[2], sys.argv[3], vehicle_num=vehicle_row)
        except ValueError:
            plot_schedule(sys.argv[1], sys.argv[2], sys.argv[3], save_to=sys.argv[4])

    # schedule, route, vehicle, vehicle row, destination
    elif len(sys.argv) == 6:
        plot_schedule(*sys.argv[1:])

    else:
        print("too many arguments! Please only provide schedule, route, vehicle and optionally vehicle row and destination")
    