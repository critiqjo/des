import json,  subprocess, os;
import matplotlib.pyplot as plt
from sys import stdout
from math import sqrt
from scipy import stats

def plot(xs, ys, xtitle, ytitle, main_title, file_name):
    fig = plt.figure(figsize=(12, 7.5), dpi=80)
    if type(ys[0]) is list:
        for y in ys:
            plt.plot(xs, y)
        plt.legend(ytitle, loc="upper left")
    else:
        plt.plot(xs, ys)
        plt.ylabel(ytitle)
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


sys_pars = dict()
runs_file = open("specific_runs.json", "r")
sys_pars = json.load(runs_file)
runs_file.close()

variable = sys_pars["variable"]
var_name = sys_pars["var_name"]
var_min = sys_pars["var_min"]
var_max = sys_pars["var_max"]
var_step = sys_pars["var_step"]
runs_per_step = sys_pars["runs_per_step"]
outdir = sys_pars["outdir"]

del sys_pars["variable"]
del sys_pars["var_name"]
del sys_pars["var_min"]
del sys_pars["var_max"]
del sys_pars["var_step"]
del sys_pars["outdir"]
del sys_pars["runs_per_step"]

cargo = subprocess.Popen(['cargo', 'build'])
cargo.wait()

try:
    os.makedirs(outdir)
except OSError:
    if not os.path.isdir(outdir):
        raise

var_pts = []
resp_times = []
utils = []
gputs = []
bputs = []
tputs = []
tfracs = []
dfracs = []

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
    temp_resp_time = 0.0

    sys_pars[variable] = x
    print variable, '=', x,
    n = runs_per_step
    for i in range(0, n):
        sim_proc = subprocess.Popen(['target/debug/des'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        res = json.loads(sim_proc.communicate(json.dumps(sys_pars))[0])
        temp_tput += res["throughput"]
        temp_gput += res["goodput"]
        temp_bput += (res["throughput"] - res["goodput"])
        temp_util += res["cpu_util"]
        temp_tfrac += res["timedout_frac"]
        temp_dfrac += res["dropped_frac"]
        temp_resp_time += res["resp_time"]
        print '.',
        stdout.flush()
    tputs.append(temp_tput/n)
    gputs.append(temp_gput/n)
    bputs.append(temp_bput/n)
    utils.append(temp_util/n)
    tfracs.append(temp_tfrac/n)
    dfracs.append(temp_dfrac/n)
    resp_times.append(temp_resp_time/n)
    print '.'

ffracs = [t + d for t, d in zip(tfracs, dfracs)]
plot(var_pts, resp_times, var_name, "Response Time", "Response Times vs. "+var_name, outdir+"resptime_"+variable+".png");
plot(var_pts, [tputs, gputs, bputs], var_name, ["Throughput", "Goodput", "Badput"],  "Throughput vs. "+var_name, outdir+"tput_"+variable+".png");
plot(var_pts, utils, var_name, "Server CPU Utilization", "CPU Utilization vs. "+var_name, outdir+"util_"+variable+".png");
plot(var_pts, [ffracs, tfracs, dfracs], var_name, ["Total failed", "Timedout", "Dropped"], "Fraction of Requests Failed vs. "+var_name, outdir+"ffracs_"+variable+".png");
print "\nComplete!"

