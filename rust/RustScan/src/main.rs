#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown, clippy::if_not_else, clippy::non_ascii_literal)]

use rustscan::benchmark::{Benchmark, NamedTimer};
use rustscan::input::{self, Config, Opts, ScriptsRequired};
use rustscan::port_strategy::PortStrategy;
use rustscan::scanner::Scanner;
use rustscan::scripts::{init_scripts, Script, ScriptFile};
use rustscan::{detail, funny_opening, output, warning};

use colorful::{Color, Colorful};
use futures::executor::block_on;
use std::collections::HashMap;
use std::net::IpAddr;
use std::string::ToString;
use std::time::Duration;

use rustscan::address::parse_addresses;

// *extern crate语句本身并不会被视为"导入"。它只是告诉Rust编译器这个crate存在。实际的导入是通过use语句完成的。
extern crate colorful;
extern crate dirs;


#[cfg(unix)]    // *cfg属性表示编译时是否包含此段代码,unix false,即Windows不启用
const DEFAULT_FILE_DESCRIPTORS_LIMIT: u64 = 8000;
// Safest batch size based on experimentation
const AVERAGE_BATCH_SIZE: u16 = 3000;

#[macro_use]    // *同时引入包中定义的宏
extern crate log;

#[cfg(not(tarpaulin_include))]  // *tarpaulin_include为false或者未定义时执行
#[allow(clippy::too_many_lines)] // *允许函数过长
/// Faster Nmap scanning with Rust
/// If you're looking for the actual scanning, check out the module Scanner
fn main() {
    env_logger::init(); // *初始化日志记录器,log包?
    let mut benchmarks = Benchmark::init(); // *初始化计时器,基准测试的东西
    let mut rustscan_bench = NamedTimer::start("RustScan"); // *计时器,开始计时,名字为RustScan

    let mut opts: Opts = Opts::read();  // *读取命令行参数
    let config = Config::read(opts.config_path.clone());    // *通过命令行传递的路径,读取配置文件
    opts.merge(&config);    // *合并配置文件和命令行参数

    debug!("Main() `opts` arguments are {:?}", opts);   // *打印opts参数

    let scripts_to_run: Vec<ScriptFile> = match init_scripts(opts.scripts) {    // *初始化脚本
        Ok(scripts_to_run) => scripts_to_run,   // *成功返回脚本
        Err(e) => { // *失败
            warning!(
                format!("Initiating scripts failed!\n{e}"),
                opts.greppable,
                opts.accessible
            );
            std::process::exit(1);  // *退出,返回1
        }
    };

    debug!("Scripts initialized {:?}", &scripts_to_run);

    if !opts.greppable && !opts.accessible {    // *如果不是grep模式,也不是access模式,哪里被改变过?
        print_opening(&opts);
    }

    let ips: Vec<IpAddr> = parse_addresses(&opts);  // *解析IP地址

    if ips.is_empty() { // *如果IP地址为空
        warning!(
            "No IPs could be resolved, aborting scan.",
            opts.greppable,
            opts.accessible
        );
        std::process::exit(1);
    }

    #[cfg(unix)]
    let batch_size: u16 = infer_batch_size(&opts, adjust_ulimit_size(&opts));   // *调整批处理大小

    #[cfg(not(unix))]   // *Windows系统
    let batch_size: u16 = AVERAGE_BATCH_SIZE;

    // Added by wasuaje - 01/26/2024:
    // exclude_ports  is an exclusion port list
    //
    let scanner = Scanner::new( // *创建扫描器
        &ips,
        batch_size,
        Duration::from_millis(opts.timeout.into()), // *超时时间
        opts.tries,
        opts.greppable,
        PortStrategy::pick(&opts.range, opts.ports, opts.scan_order),
        opts.accessible,
        opts.exclude_ports.unwrap_or_default(),
    );
    debug!("Scanner finished building: {:?}", scanner);

    let mut portscan_bench = NamedTimer::start("Portscan"); // *计时器,开始计时,名字为Portscan
    let scan_result = block_on(scanner.run());  // *扫描器运行
    portscan_bench.end();   // *计时器,结束计时
    benchmarks.push(portscan_bench);    // *将计时器放入benchmarks

    let mut ports_per_ip = HashMap::new();  // *创建HashMap,ip地址和对应的端口号

    for socket in scan_result { // *遍历扫描结果,将端口号和ip地址放入HashMap
        ports_per_ip
            .entry(socket.ip())
            .or_insert_with(Vec::new)
            .push(socket.port());
    }

    for ip in ips {
        if ports_per_ip.contains_key(&ip) {
            continue;
        }

        // If we got here it means the IP was not found within the HashMap, this
        // means the scan couldn't find any open ports for it.

        let x = format!("Looks like I didn't find any open ports for {:?}. This is usually caused by a high batch size.
        \n*I used {} batch size, consider lowering it with {} or a comfortable number for your system.
        \n Alternatively, increase the timeout if your ping is high. Rustscan -t 2000 for 2000 milliseconds (2s) timeout.\n",
        ip,
        opts.batch_size,
        "'rustscan -b <batch_size> -a <ip address>'");
        warning!(x, opts.greppable, opts.accessible);
    }   // *如果ip地址没有找到,说明扫描没有找到任何开放的端口

    let mut script_bench = NamedTimer::start("Scripts");    // *计时器,开始计时,名字为Scripts
    for (ip, ports) in &ports_per_ip {
        let vec_str_ports: Vec<String> = ports.iter().map(ToString::to_string).collect();

        // nmap port style is 80,443. Comma separated with no spaces.
        let ports_str = vec_str_ports.join(",");

        // if option scripts is none, no script will be spawned
        if opts.greppable || opts.scripts == ScriptsRequired::None {
            println!("{} -> [{}]", &ip, ports_str);
            continue;
        }
        detail!("Starting Script(s)", opts.greppable, opts.accessible);

        // Run all the scripts we found and parsed based on the script config file tags field.
        for mut script_f in scripts_to_run.clone() {
            // This part allows us to add commandline arguments to the Script call_format, appending them to the end of the command.
            if !opts.command.is_empty() {
                let user_extra_args = &opts.command.join(" ");
                debug!("Extra args vec {:?}", user_extra_args);
                if script_f.call_format.is_some() {
                    let mut call_f = script_f.call_format.unwrap();
                    call_f.push(' ');
                    call_f.push_str(user_extra_args);
                    output!(
                        format!("Running script {:?} on ip {}\nDepending on the complexity of the script, results may take some time to appear.", call_f, &ip),
                        opts.greppable,
                        opts.accessible
                    );
                    debug!("Call format {}", call_f);
                    script_f.call_format = Some(call_f);
                }
            }

            // Building the script with the arguments from the ScriptFile, and ip-ports.
            let script = Script::build(
                script_f.path,
                *ip,
                ports.clone(),
                script_f.port,
                script_f.ports_separator,
                script_f.tags,
                script_f.call_format,
            );
            match script.run() {
                Ok(script_result) => {
                    detail!(script_result.to_string(), opts.greppable, opts.accessible);
                }
                Err(e) => {
                    warning!(&format!("Error {e}"), opts.greppable, opts.accessible);
                }
            }
        }
    }   // *运行一组脚本,并打印结果

    // To use the runtime benchmark, run the process as: RUST_LOG=info ./rustscan
    // *要使用运行时基准，请运行该流程： RUST_LOG=info ./rustscan
    script_bench.end();
    benchmarks.push(script_bench);
    rustscan_bench.end();
    benchmarks.push(rustscan_bench);
    debug!("Benchmarks raw {:?}", benchmarks);
    info!("{}", benchmarks.summary());
}

/// Prints the opening title of RustScan
// *打印RustScan的开头标题
#[allow(clippy::items_after_statements, clippy::needless_raw_string_hashes)]    // *允许之后定义的函数,允许不必要的原始字符串哈希
fn print_opening(opts: &Opts) {
    debug!("Printing opening");
    let s = format!(
        "{}\n{}\n{}\n{}\n{}",
        r#".----. .-. .-. .----..---.  .----. .---.   .--.  .-. .-."#,
        r#"| {}  }| { } |{ {__ {_   _}{ {__  /  ___} / {} \ |  `| |"#,
        r#"| .-. \| {_} |.-._} } | |  .-._} }\     }/  /\  \| |\  |"#,
        r#"`-' `-'`-----'`----'  `-'  `----'  `---' `-'  `-'`-' `-'"#,
        r#"The Modern Day Port Scanner."#
    );
    println!("{}", s.gradient(Color::Green).bold());    // *打印绿色的标题
    let info = format!(
        "{}\n{}\n{}\n{}",
        r#"________________________________________"#,
        r#": http://discord.skerritt.blog         :"#,
        r#": https://github.com/RustScan/RustScan :"#,
        r#" --------------------------------------"#
    );
    println!("{}", info.gradient(Color::Yellow).bold());
    funny_opening!();

    // *打印配置文件路径
    let config_path = opts
        .config_path
        .clone()
        .unwrap_or_else(input::default_config_path);

    // *调试的玩意
    detail!(
        format!("The config file is expected to be at {config_path:?}"),
        opts.greppable,
        opts.accessible
    );
}

// *后面都是些不重要的东西,主要是Linux的一些设置,和一些单元测试
#[cfg(unix)]
fn adjust_ulimit_size(opts: &Opts) -> u64 {
    use rlimit::Resource;

    if let Some(limit) = opts.ulimit {
        if Resource::NOFILE.set(limit, limit).is_ok() {
            detail!(
                format!("Automatically increasing ulimit value to {limit}."),
                opts.greppable,
                opts.accessible
            );
        } else {
            warning!(
                "ERROR. Failed to set ulimit value.",
                opts.greppable,
                opts.accessible
            );
        }
    }

    let (soft, _) = Resource::NOFILE.get().unwrap();
    soft
}

#[cfg(unix)]
fn infer_batch_size(opts: &Opts, ulimit: u64) -> u16 {
    use std::convert::TryInto;

    let mut batch_size: u64 = opts.batch_size.into();

    // Adjust the batch size when the ulimit value is lower than the desired batch size
    if ulimit < batch_size {
        warning!("File limit is lower than default batch size. Consider upping with --ulimit. May cause harm to sensitive servers",
            opts.greppable, opts.accessible
        );

        // When the OS supports high file limits like 8000, but the user
        // selected a batch size higher than this we should reduce it to
        // a lower number.
        if ulimit < AVERAGE_BATCH_SIZE.into() {
            // ulimit is smaller than aveage batch size
            // user must have very small ulimit
            // decrease batch size to half of ulimit
            warning!("Your file limit is very small, which negatively impacts RustScan's speed. Use the Docker image, or up the Ulimit with '--ulimit 5000'. ", opts.greppable, opts.accessible);
            info!("Halving batch_size because ulimit is smaller than average batch size");
            batch_size = ulimit / 2;
        } else if ulimit > DEFAULT_FILE_DESCRIPTORS_LIMIT {
            info!("Batch size is now average batch size");
            batch_size = AVERAGE_BATCH_SIZE.into();
        } else {
            batch_size = ulimit - 100;
        }
    }
    // When the ulimit is higher than the batch size let the user know that the
    // batch size can be increased unless they specified the ulimit themselves.
    else if ulimit + 2 > batch_size && (opts.ulimit.is_none()) {
        detail!(format!("File limit higher than batch size. Can increase speed by increasing batch size '-b {}'.", ulimit - 100),
        opts.greppable, opts.accessible);
    }

    batch_size
        .try_into()
        .expect("Couldn't fit the batch size into a u16.")
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::{adjust_ulimit_size, infer_batch_size};
    use super::{print_opening, Opts};

    #[test]
    #[cfg(unix)]
    fn batch_size_lowered() {
        let mut opts = Opts::default();
        opts.batch_size = 50_000;
        let batch_size = infer_batch_size(&opts, 120);

        assert!(batch_size < opts.batch_size);
    }

    #[test]
    #[cfg(unix)]
    fn batch_size_lowered_average_size() {
        let mut opts = Opts::default();
        opts.batch_size = 50_000;
        let batch_size = infer_batch_size(&opts, 9_000);

        assert!(batch_size == 3_000);
    }
    #[test]
    #[cfg(unix)]
    fn batch_size_equals_ulimit_lowered() {
        // because ulimit and batch size are same size, batch size is lowered
        // to ULIMIT - 100
        let mut opts = Opts::default();
        opts.batch_size = 50_000;
        let batch_size = infer_batch_size(&opts, 5_000);

        assert!(batch_size == 4_900);
    }
    #[test]
    #[cfg(unix)]
    fn batch_size_adjusted_2000() {
        // ulimit == batch_size
        let mut opts = Opts::default();
        opts.batch_size = 50_000;
        opts.ulimit = Some(2_000);
        let batch_size = adjust_ulimit_size(&opts);

        assert!(batch_size == 2_000);
    }

    #[test]
    #[cfg(unix)]
    fn test_high_ulimit_no_greppable_mode() {
        let mut opts = Opts::default();
        opts.batch_size = 10;
        opts.greppable = false;

        let batch_size = infer_batch_size(&opts, 1_000_000);

        assert!(batch_size == opts.batch_size);
    }

    #[test]
    fn test_print_opening_no_panic() {
        let mut opts = Opts::default();
        opts.ulimit = Some(2_000);
        // print opening should not panic
        print_opening(&opts);
    }
}
