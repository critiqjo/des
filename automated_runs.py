import json,  subprocess, os;
import matplotlib.pyplot as plt
from math import sqrt
from scipy import stats

def plot(xs, ys, xtitle, ytitle,main_title, file_name, scatter=False, errorbar=False, error_rate=0.0, intervals=[]):
    fig = plt.figure()
    if scatter:
        plt.scatter(xs,ys)
        if errorbar:
            plt.errorbar(xs, ys, yerr=intervals, linestyle="None")
    else:
        plt.plot(xs, ys)
    plt.ylim([0,max(ys)*1.2])
    plt.xlabel(xtitle)
    plt.ylabel(ytitle)
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

def compute_ci(sys, variable, error_rate, iters):
    vec = []
    sys[variable] = 40
    for i in range(1, iters+1):
        sim_run = subprocess.Popen(['target/debug/des'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        res = json.loads(sim_run.communicate(json.dumps(sys))[0])
        vec.append(res[variable])
    resp_lb, resp_ub = conf_ivals(vec, error_rate)
    return resp_lb, mean(vec), resp_ub


sys = dict()
multi_run_file = open("auto_runs.json", "r")
sys = json.load(multi_run_file)
multi_run_file.close()

n_users_start = sys["n_users_start"]
n_users_end = sys["n_users_end"]
del sys["n_users_start"]
del sys["n_users_end"]

cargo = subprocess.Popen(['cargo', 'build'])
cargo.wait()


#Simulation with varied number of users
n_users = []
resp_times = []
utils = []
gputs = []
bputs = []
tputs = []
tfracs = []
dfracs = []
drates = []
ivals = []
error_rate = 0.05

for x in range(n_users_start, n_users_end, 2):
    n_users.append(x)
    temp_tput = 0.0
    temp_gput = 0.0
    temp_bput = 0.0
    temp_util = 0.0
    temp_tfrac = 0.0
    temp_dfrac = 0.0
    temp_drate = 0.0
    temp_resp_time = 0.0
    sys["n_users"] = x
    for i in range(0,3):
        sim_run = subprocess.Popen(['target/debug/des'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        res = json.loads(sim_run.communicate(json.dumps(sys))[0])
        temp_tput += res["throughput"]
        temp_gput += res["goodput"]
        temp_bput += (res["throughput"] - res["goodput"])
        temp_util += res["cpu_util"]
        temp_tfrac += res["timedout_frac"]
        temp_dfrac += res["dropped_frac"]
        temp_drate += res["drop_rate"]
        temp_resp_time += res["resp_time"]

    (resp_lb, resp_mean, resp_ub) = compute_ci(sys, "resp_time", error_rate, 30)
    ivals.append((resp_ub-resp_lb)/2)
    print "Average response time ",resp_mean, " lies with ", (1-error_rate)*100,"% confidence in interval (", resp_lb, ",", resp_ub, ")"
    tputs.append(temp_tput/3.0)
    gputs.append(temp_gput/3.0)
    bputs.append(temp_bput/3.0)
    utils.append(temp_util/3.0)
    tfracs.append(temp_tfrac/3.0)
    dfracs.append(temp_dfrac/3.0)
    drates.append(temp_drate/3.0)
    resp_times.append(resp_mean)

plot(n_users, resp_times, "Number of Users", "Response Time", "Response Times Vs. Number of Users ", "nusers_v_resp.png",True, True, error_rate, ivals)
exit
plot(n_users, gputs, "Number of Users", "Goodput",  "Goodput Vs. Number of Users", "gput_v_nusers.png");
plot(n_users, bputs, "Number of Users", "Badput",  "Badput Vs. Number of Users", "bput_v_nusers.png");
plot(n_users, tputs, "Number of Users", "Throughput",  "Througput Vs. Number of Users ", "tput_v_nusers.png");
plot(n_users, utils, "Number of Users", "Server CPU Utilization", "CPU Utilization Vs. Number of Users", "util_v_nusers.png");
plot(tputs, resp_times, "Throughput", "Response Time", "Response Time Vs. Throughput", "resp_v_tput.png", True);
plot(tputs, utils, "Throughput", "Server CPU Utilization", "CPU Utilization Vs. Throughput", "util_v_tput.png");
plot(tputs, utils, "Throughput", "Server CPU Utilization", "CPU Utilization Vs. Throughput", "util_v_tput.png", True);
plot(n_users, tfracs, "Number of Users", "Fraction of Requests Timeout", "Fraction of Requests Timedout Vs. Number of Users", "tfracs_v_nusers.png");
plot(n_users, dfracs, "Number of Users", "Fraction of Requests Dropped", "Fraction of Requests Failed Vs. Number of Users", "dfracs_v_nusers.png");
plot(n_users, drates, "Number of Users", "Drop Rate", "Drop Rate Vs. Number of Users", "drate_v_nusers.png");
print "Plots are generated"

