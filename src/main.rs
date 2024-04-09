extern crate num;
extern crate sysinfo;
#[macro_use]
extern crate num_derive;

use std::fmt;
use std::thread;
use std::time;

// Add whitespace prepending a value
fn add_whitespace(str_to_format: String, chars_tot: u64) -> String {
    // Get the length of the passed string and calculate how many spaces to add
    let char_num = str_to_format.as_bytes().len() as u64;
    let space_num = chars_tot - char_num;

    // Create a new string to add everything to
    let mut ret_string = String::new();

    // Add all the needed spaces to that string
    for _i in 0..space_num {
        ret_string.push(' ');
    }

    // Add the original string to it
    ret_string.push_str(&str_to_format);

    ret_string
}

// Get the average core usage
fn get_cpu_use(req_sys: &sysinfo::System) -> f32 {
    // Put all of the core loads into a vector
    let mut cpus: Vec<f32> = Vec::new();
    for core in req_sys.cpus() {
        cpus.push(core.cpu_usage());
    }

    // Get the average load
    let cpu_tot: f32 = cpus.iter().sum();
    let cpu_avg: f32 = cpu_tot / cpus.len() as f32;

    cpu_avg
}

// Divide the used RAM by the total RAM
fn get_ram_use(req_sys: &sysinfo::System) -> f32 {
    (req_sys.used_memory() as f32) / (req_sys.total_memory() as f32) * 100.
}

// Get the total network (down) usage
fn get_ntwk_dwn(req_net: &sysinfo::Networks) -> u64 {
    // Get the total bytes recieved by every network interface
    let mut rcv_tot: Vec<u64> = Vec::new();
    for (_interface_name, ntwk) in req_net {
        rcv_tot.push(ntwk.received() as u64);
    }

    rcv_tot.iter().sum()
    //let ntwk_processed = (ntwk_tot / 128) as i32;
}

// Get the total network (up) usage
fn get_ntwk_up(req_net: &sysinfo::Networks) -> u64 {
    // Get the total bytes recieved by every network interface
    let mut snd_tot: Vec<u64> = Vec::new();
    for (_interface_name, ntwk) in req_net {
        //println!("{_interface_name}");
        snd_tot.push(ntwk.transmitted() as u64);
    }

    snd_tot.iter().sum()
}

fn main() {
    // Define a system that we will check
    let mut current_sys = sysinfo::System::new_all();
    let mut current_net = sysinfo::Networks::new_with_refreshed_list();
    let dur: u64 = 1000;

    loop {
        // Refresh the system
        current_sys.refresh_all();
        current_net.refresh();

        // Call each function to get all the values we need
        let cpu_avg = get_cpu_use(&current_sys);
        let ram_prcnt = get_ram_use(&current_sys);
        let ntwk_dwn = get_ntwk_dwn(&current_net);
        let ntwk_up = get_ntwk_up(&current_net);

        let prnt_cpu = add_whitespace(format!("{:.1} %", cpu_avg), 7);
        let prnt_ram = add_whitespace(format!("{:.1} %", ram_prcnt), 7);
        let prnt_down = add_whitespace(to_pretty_bytes(ntwk_dwn, dur), 10);
        let prnt_up = add_whitespace(to_pretty_bytes(ntwk_up, dur), 10);

        println!(
            "[RAM:{}] [CPU:{}] [DOWN:{}] [UP:{}]",
            prnt_ram,
            prnt_cpu,
            prnt_down, //to_pretty_bytes(ntwk_dwn, dur),
            prnt_up    //to_pretty_bytes(ntwk_up, dur)
        );

        // Wait one second
        thread::sleep(time::Duration::from_millis(dur));
    }
}

#[derive(Debug, FromPrimitive)]
enum DataUnit {
    B = 0,
    K = 1,
    M = 2,
    G = 3,
    T = 4,
    P = 5,
    E = 6,
    Z = 7,
    Y = 8,
}

impl fmt::Display for DataUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn to_pretty_bytes(input_in_bytes: u64, timespan_in_ms: u64) -> String {
    if input_in_bytes < 1024 {
        return format!("0 KB");
    }

    let seconds = timespan_in_ms as f32 / f32::powf(10.0, 3.0);
    let magnitude = input_in_bytes.ilog(1024);
    let base: Option<DataUnit> = num::FromPrimitive::from_u32(magnitude);
    let result = (input_in_bytes as f32 / seconds) / ((1 as u64) << (magnitude * 10)) as f32;

    match base {
        Some(DataUnit::B) => format!("{result:.2} B"),
        Some(unit) => format!("{result:.2} {unit}B"),
        None => format!("Unknown data unit"),
    }
}

fn to_pretty_bits(input_in_bytes: u64, timespan_in_ms: u64) -> String {
    if input_in_bytes < 1000 {
        return format!("0 Kb");
    }
   
    let input = input_in_bytes * 8;

    let seconds = timespan_in_ms as f32 / f32::powf(10.0, 3.0);
    let magnitude = input.ilog(1000);
    let base: Option<DataUnit> = num::FromPrimitive::from_u32(magnitude);
    let result = (input as f32 / seconds) / f32::powf(1000.0, magnitude as f32);

    match base {
        Some(DataUnit::B) => format!("{result:.2} b"),
        Some(unit) => format!("{result:.2} {unit}b"),
        None => format!("Unknown data unit"),
    }
}

/*
        public enum SizeSuffix : int { bytes = 0, KB, MB, GB, TB, PB, EB, ZB, YB }

        public class PrettySize : Tuple<decimal, SizeSuffix>
        {
            public PrettySize(decimal key, SizeSuffix value) : base(key, value) { }

            public override string ToString() =>
                $"{Item1:n1} {Item2}";
        }

        /// <summary> Gets the pretty representation of {source} as a size </summary>
        public static PrettySize ToPrettySize(this Int64 source, SizeSuffix? valueSuffix = SizeSuffix.bytes, SizeSuffix? newSuffix = null)
        {
            if (source < 0)
            {
                var kvp = ToPrettySize(-source, valueSuffix, newSuffix);
                return new PrettySize(kvp.Item1 * -1, kvp.Item2);
            }
            if (source == 0)
            {
                return new PrettySize(0.0M, SizeSuffix.bytes);
            }

            // find the next suitable size suffix
            if (!newSuffix.HasValue)
                newSuffix = valueSuffix.Value.Next((int)Math.Log(source, 1024));

            int mag = (int)newSuffix;
            decimal adjustedSize;

            if (valueSuffix.HasValue)
            {
                int steps = (int)valueSuffix - mag;

                if (steps > 0)
                    adjustedSize = (decimal)source * (1L << (steps * 10));
                else
                    adjustedSize = (decimal)source / (1L << (Math.Abs(steps) * 10));
            }
            // convert from bytes
            else
                adjustedSize = (decimal)source / (1L << (mag * 10)); // 64 bit signed int

            return new PrettySize(decimal.Round(adjustedSize, 2), mag.ToEnum<SizeSuffix>());
        }
*/
