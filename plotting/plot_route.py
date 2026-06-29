import sys
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.colors
from matplotlib.cm import ScalarMappable
import os.path

GLOBAL_V_MAX = 180 # [km/h]


def load_route(route_name, repeat=1):
    # add .csv file ending if not provided
    if os.path.splitext(route_name)[1] == "":
        route_name = route_name + ".csv"

    route = np.loadtxt(route_name, skiprows=1, delimiter=',')

    lengths = np.repeat(route[:, 0], repeat) / repeat # in m, divide by repeat so total length stays the same
    slopes = np.repeat(route[:, 1], repeat) # in %
    max_speeds = np.repeat(route[:, 2] / 3.6, repeat) # convert to m/s

    return (lengths, slopes, max_speeds)


def plot_route(path, save_to=None, cmap_name="Spectral"):
    lengths, slopes, max_speeds = load_route(path)
    max_speeds = max_speeds * 3.6 # [km/h]
    cmap = plt.get_cmap(cmap_name)
    
    # retrieve route name from path
    tail = os.path.split(path)[1]
    route_name = os.path.splitext(tail)[0]

    x = np.concatenate(([0.], np.cumsum(lengths)))
    heights = np.concatenate(([0.], np.cumsum(lengths * slopes / 100)))

    fig, ax = plt.subplots(layout="constrained")
    plt.plot(x, heights)
    plt.title(f"Overview {route_name}")
    plt.xlabel("distance [m]")
    plt.ylabel("elevation [m]")

    # extend plot below to allow for visible filling even where the curve is at its minimum
    base_level = np.min(heights) - 0.02 * (np.max(heights) - np.min(heights))

    # show maximum velocity colorcoded below the curve
    for i in range(len(x)-1):
        plt.fill_between(x[i:i+2], heights[i:i+2], base_level, color=cmap(max_speeds[i] / GLOBAL_V_MAX))
    fig.colorbar(ScalarMappable(norm=matplotlib.colors.Normalize(0, GLOBAL_V_MAX), cmap=cmap_name), ax=ax, label="speed [km/h]")

    # set default name if no destination given
    if save_to == None:
        save_to = f"plotting/figures/overview_{route_name}.png"

    # add .png if destination has no file extension
    elif os.path.splitext(save_to)[1] == "":
        save_to = save_to + ".png"

    # use given destination as-is
    else:
        pass

    plt.savefig(save_to)

    print(route_name, "successfully saved to", save_to)


if __name__ == "__main__":
    print("executing plot_route.py")

    if len(sys.argv) <= 1:
        print("not enough arguments! Please provide a file name and optionally a destination")

    elif len(sys.argv) == 2:
        plot_route(sys.argv[1], None)

    elif len(sys.argv) == 3:
        plot_route(sys.argv[1], sys.argv[2])

    else:
        print("too many arguments! Please only provide a file name and optionally a destination")
    