import json,  subprocess, os;
import matplotlib.pyplot as plt
from sys import stdout
from math import sqrt
from scipy import stats

def plot(xs, ys, xtitle, ytitle, main_title, file_name):
    fig = plt.figure()
    if type(ys[0]) is list:
        for y in ys:
            plt.plot(xs, y)
        plt.legend(ytitle)
    else:
        plt.plot(xs, ys)
        plt.ylabel(ytitle)
        #plt.ylim([0,max(ys)*1.2])
    plt.xlabel(xtitle)
    plt.title(main_title)
    plt.savefig(file_name, bbox_inches='tight')
    plt.close(fig)

def mean(samples):
    return sum(samples)/len(samples)

def std(samples):
    n = len(samples)-1
    m = sum(samples)/n
    variance = [(pow((x-m), 2))/n for x in samples]
    std = [sqrt(x) for x in variance]
    return sum(std)

def conf_ivals(samples, error):
    n = len(samples)
    m = mean(samples)
    sd = std(samples)
    alpha = stats.norm.ppf(1 - error/2)
    return (m - sd*alpha/sqrt(n), m + sd*alpha/sqrt(n))


simsys = dict()
runs_file = open("specific_runs.json", "r")
simsys = json.load(runs_file)
runs_file.close()

variable = simsys["variable"]
var_name = simsys["var_name"]
var_min = simsys["var_min"]
var_max = simsys["var_max"]
var_step = simsys["var_step"]
outdir = simsys["outdir"]

del simsys["variable"]
del simsys["var_name"]
del simsys["var_min"]
del simsys["var_max"]
del simsys["var_step"]
del simsys["outdir"]

cargo = subprocess.Popen(['cargo', 'build'])
cargo.wait()

var_pts = []
resp_times = []
utils = []
gputs = []
bputs = []
tputs = []
tfracs = []
dfracs = []
drates = []

def frange(x, y, step):
    while x < y:
        yield x
        x += step

for x in frange(var_min, var_max, var_step):
    var_pts.append(x)
    temp_tput = 0.0
    temp_gput = 0.0
    temp_bput = 0.0
    temp_util = 0.0
    temp_tfrac = 0.0
    temp_dfrac = 0.0
    temp_drate = 0.0
    temp_resp_time = 0.0

    simsys[variable] = x
    for i in range(0,3):
        sim_run = subprocess.Popen(['target/debug/des'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        res = json.loads(sim_run.communicate(json.dumps(simsys))[0])
        temp_tput += res["throughput"]
        temp_gput += res["goodput"]
        temp_bput += (res["throughput"] - res["goodput"])
        temp_util += res["cpu_util"]
        temp_tfrac += res["timedout_frac"]
        temp_dfrac += res["dropped_frac"]
        temp_resp_time += res["resp_time"]
    tputs.append(temp_tput/3.0)
    gputs.append(temp_gput/3.0)
    bputs.append(temp_bput/3.0)
    utils.append(temp_util/3.0)
    tfracs.append(temp_tfrac/3.0)
    dfracs.append(temp_dfrac/3.0)
    resp_times.append(temp_resp_time/3.0)
    print('.'),
    stdout.flush()

ffracs = [t + d for t, d in zip(tfracs, dfracs)]
plot(var_pts, resp_times, var_name, "Response Time", "Response Times vs. "+var_name, outdir+"resptime_"+variable+".png");
plot(var_pts, [tputs, gputs, bputs], var_name, ["Throughput", "Goodput", "Badput"],  "Throughput vs. "+var_name, outdir+"tput_"+variable+".png");
plot(var_pts, utils, var_name, "Server CPU Utilization", "CPU Utilization vs. "+var_name, outdir+"util_"+variable+".png");
plot(var_pts, [ffracs, tfracs, dfracs], var_name, ["Total failed", "Timedout", "Dropped"], "Fraction of Requests Failed vs. "+var_name, outdir+"ffracs_"+variable+".png");
print "Plots are generated"

