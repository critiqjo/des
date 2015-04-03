import json, subprocess, os;
import matplotlib.pyplot as plt
import numpy as np
from sys import stdout
from math import sqrt
from scipy import stats

basedir = os.path.dirname(os.path.abspath(__file__)) + "/"
def plot(xs, ys, xtitle, ytitle, main_title, file_name, plttype="plot", yerrs=[]):
    fig = plt.figure(figsize=(12, 7.5), dpi=80)
    if plttype=="scatter":
        plt.scatter(xs, ys, marker="x")
    elif plttype=="errorbar":
        plt.errorbar(xs, ys, yerr=yerrs, fmt="-o")
    elif plttype=="logerrorbar":
        plt.errorbar(xs, ys, yerr=yerrs, fmt="-o")
        plt.yscale("log")
        p2 = np.exp2(np.arange( np.ceil(np.log2(max(ys))) + 1 ))
        plt.yticks(p2, p2)
    elif type(ys[0]) is list:
        for y in ys:
            plt.plot(xs, y)
        plt.legend(ytitle, loc="upper left")
    else:
        plt.plot(xs, ys)
        plt.ylabel(ytitle)
    plt.xlabel(xtitle)
    plt.title(main_title)
    plt.savefig(file_name, bbox_inches="tight")
    plt.close(fig)

def mean(samples):
    return sum(samples)/len(samples)

def std(samples):
    n = len(samples)-1
    m = sum(samples)/n
    variance = [(pow((x-m), 2))/n for x in samples]
    std = [sqrt(x) for x in variance]
    return sum(std)

def conf_ival(samples, error):
    n = len(samples)
    m = mean(samples)
    sd = std(samples)
    alpha = stats.norm.ppf(1 - error/2)
    return (m, sd*alpha/sqrt(n))

des = os.path.relpath(basedir + "../target/debug/des")
def sim_run(sys_pars):
    sim_proc = subprocess.Popen([des], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    return json.loads(sim_proc.communicate(json.dumps(sys_pars))[0])

sys_pars = dict()
runs_file = open(basedir + "auto_pars.json", "r")
sys_pars = json.load(runs_file)
runs_file.close()

n_users_start = sys_pars["n_users_start"]
n_users_end = sys_pars["n_users_end"]
n_users_step = sys_pars["n_users_step"]
runs_per_step = sys_pars["runs_per_step"]
del sys_pars["n_users_start"]
del sys_pars["n_users_end"]
del sys_pars["n_users_step"]
del sys_pars["runs_per_step"]

n_users = []
resp_times = []
utils = []
gputs = []
bputs = []
tputs = []
tfracs = []
dfracs = []
drates = []
resp_errs = []

all_tputs = []
all_resps = []
all_utils = []

conf_error = 0.05

for x in range(n_users_start, n_users_end, n_users_step):
    n_users.append(x)
    temp_tput = 0.0
    temp_gput = 0.0
    temp_bput = 0.0
    temp_util = 0.0
    temp_tfrac = 0.0
    temp_dfrac = 0.0
    temp_drate = 0.0
    temp_resp = []

    sys_pars["n_users"] = x
    print "n_users =", x,
    n = runs_per_step
    for i in range(0, n):
        res = sim_run(sys_pars)
        temp_tput += res["throughput"]
        temp_gput += res["goodput"]
        temp_bput += (res["throughput"] - res["goodput"])
        temp_util += res["cpu_util"]
        temp_tfrac += res["timedout_frac"]
        temp_dfrac += res["dropped_frac"]
        temp_drate += res["drop_rate"]
        temp_resp.append(res["resp_time"])

        all_tputs.append(res["throughput"])
        all_resps.append(res["resp_time"])
        all_utils.append(res["cpu_util"])

        print ".",
        stdout.flush()

    tputs.append(temp_tput/n)
    gputs.append(temp_gput/n)
    bputs.append(temp_bput/n)
    utils.append(temp_util/n)
    tfracs.append(temp_tfrac/n)
    dfracs.append(temp_dfrac/n)
    drates.append(temp_drate/n)
    (resp_mean, resp_err) = conf_ival(temp_resp, conf_error)
    resp_times.append(resp_mean)
    resp_errs.append(resp_err)
    print "\nAverage response time =", resp_mean, u"\xb1", resp_err, "with", (1-conf_error)*100, "% confidence"

ffracs = [t + d for t, d in zip(tfracs, dfracs)]

xlabel = "Number of Users"
plot(n_users, resp_times, xlabel, "Response Time", "Response Times vs. "+xlabel, "resp_nusers.png", "errorbar", resp_errs)
plot(n_users, resp_times, xlabel, "Response Time", "Response Times vs. "+xlabel, "resp_nusers_log.png", "logerrorbar", resp_errs)
plot(n_users, [tputs, gputs, bputs], xlabel, ["Throughput", "Goodput", "Badput"],  "Throughput vs. "+xlabel, "tput_nusers.png");
plot(n_users, utils, xlabel, "CPU Utilization", "CPU Utilization vs. "+xlabel, "util_nusers.png");
plot(n_users, [ffracs, tfracs, dfracs], xlabel, ["Total failed", "Timedout", "Dropped"], "Fraction of Requests Failed vs. "+xlabel, "ffracs_nusers.png");
plot(n_users, drates, xlabel, "Drop Rate", "Drop Rate vs. "+xlabel, "drate_nusers.png");
plot(all_tputs, all_resps, "Throughput", "Response Time", "Response Time vs. Throughput", "resp_tput.png", "scatter");
plot(all_tputs, all_utils, "Throughput", "Server CPU Utilization", "CPU Utilization vs. Throughput", "util_tput.png", "scatter");
print "\nComplete!"

