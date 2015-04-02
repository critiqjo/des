import json,  subprocess, os;
import matplotlib.pyplot as plt
from math import sqrt
from scipy import stats

def plot(xs, ys, xtitle, ytitle,main_title, file_name, scatter=False):
    fig = plt.figure()
    if scatter:
        plt.scatter(xs,ys)
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

#Simulation for Computing Condidence Interval for Reponse Time with Error Rate 5%
error_rate = 0.05
r_times = []
sys["n_users"] = 40
for i in range(1, 30):
    sim_run = subprocess.Popen(['target/debug/des'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    res = json.loads(sim_run.communicate(json.dumps(sys))[0])
    r_times.append(res["resp_time"])
resp_lb, resp_ub = conf_ivals(r_times, error_rate)
print "\n95% Confidence Interval of Response Time: ", resp_lb, resp_ub

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

    for i in range(0,3):
        sys["n_users"] = x
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
    tputs.append(temp_tput/3.0)
    gputs.append(temp_gput/3.0)
    bputs.append(temp_bput/3.0)
    utils.append(temp_util/3.0)
    tfracs.append(temp_tfrac/3.0)
    dfracs.append(temp_dfrac/3.0)
    drates.append(temp_drate/3.0)
    resp_times.append(temp_resp_time/3.0)

plot(n_users, resp_times, "Number of Users", "Response Time", "Number of Users Vs. Response Times", "nusers_v_resp.png");
plot(n_users, gputs, "Number of Users", "Goodput",  "Goodput Vs. Number of Users", "gput_v_nusers.png");
plot(n_users, bputs, "Number of Users", "Badput",  "Badput Vs. Number of Users", "bput_v_nusers.png");
plot(n_users, tputs, "Number of Users", "Throughput",  "Througput Vs. Number of Users ", "tput_v_nusers.png");
plot(n_users, utils, "Number of Users", "Server CPU Utilization", "CPU Utilization Vs. Number of Users", "util_v_util.png");
plot(tputs, resp_times, "Throughput", "Response Time", "Response Time Vs. Throughput", "resp_v_tput.png", True);
plot(tputs, utils, "Throughput", "Server CPU Utilization", "CPU Utilization Vs. Throughput", "util_v_tput.png");
plot(tputs, utils, "Throughput", "Server CPU Utilization", "CPU Utilization Vs. Throughput", "util_v_tput.png", True);
plot(n_users, tfracs, "Number of Users", "Fraction of Requests Timeout", "Fraction of Requests Timedout Vs. Number of Users", "tfracs_v_nusers.png");
plot(n_users, dfracs, "Number of Users", "Fraction of Requests Dropped", "Fraction of Requests Failed Vs. Number of Users", "dfracs_v_nusers.png");
plot(n_users, drates, "Number of Users", "Drop Rate", "Drop Rate Vs. Number of Users", "drate_v_nusers.png");
print "Plots are generated"

