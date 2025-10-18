// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, os::windows::process, thread, time::Duration, collections::HashMap};
use slint::ComponentHandle;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System, Motherboard, Pid};

mod timeConvert;
use timeConvert::convert_time;



slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {

    let mut sys = System::new_all();
    sys.refresh_all();
    // let disks = Disks::new_with_refreshed_list();
    // // let dk_usage = disks.list()[0].usage();
    // // println!("{:?}", dk_usage);
    // for disk in &disks{
    //     println!("{:?}",disk.usage());
    // }

    let board_info = if let Some(m) = Motherboard::new() {
        let info = format!("{} ({})", m.vendor_name().unwrap_or("N/A".to_string()), m.name().unwrap_or("N/A".to_string()));
        println!("{}", info);
        info
    } else {
        String::from("N/A")
    };


    let os_name= System::name().unwrap_or("default".to_string());
    let os_ver = System::os_version().unwrap_or("default".to_string());
    let cpu_brand = sys.cpus()[0].brand();
    let core_count = System::physical_core_count().unwrap();
    let mut thread_count = 0;
    for i in sys.cpus(){
        thread_count += 1;
    }

    let total_ram = sys.total_memory().div_ceil(1024*1024*1024)as i32;
    println!("RAM: {}", total_ram);

    println!("Name: {}", os_name);
    println!("Brand: {}",cpu_brand);
    println!("Core: {}", core_count);
    println!("Thread: {}", thread_count);

    let cpu_count = format!("{core_count}({thread_count})");
    println!("Both: {}", cpu_count);

    println!("***************************");

    for (pid, process) in sys.processes(){
        println!("{} {:?}", pid, process.name());
    }

    println!("***************************");

    let mut totals: HashMap<String, u64> = HashMap::new();
    for (_, proc_) in sys.processes() {
        let name = proc_.name().to_string_lossy().to_string();
        let mem = proc_.memory();

        *totals.entry(name).or_insert(0) += mem;
    }

    let mut process_totals: Vec<(String, u64)> = totals.into_iter().collect();
    
    process_totals.sort_by(|a, b| b.1.cmp(&a.1));

    for (name, mem) in process_totals.iter().take(20) {
        println!("{:<30} | {:>8}KB", name, mem);
    }

    let main_window = MainWindow::new()?;
    main_window.set_osName(os_name.into());
    main_window.set_osVersion(os_ver.into());
    main_window.set_cpuName(cpu_brand.into());
    main_window.set_cpuCount(cpu_count.into());
    main_window.set_motherBoard(board_info.into());
    main_window.set_memory(total_ram);

    let handle = main_window.as_weak();

    let handle_cpu_mem = handle.clone();
    std::thread::spawn(move || {
        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let cpu_global = (sys.global_cpu_usage()*10.0).round()/10.0;
            let mem_used = ((sys.used_memory() as f32/1073741824.0)*10.0).round()/10.0;
            let mem_total = ((sys.total_memory() as f32/1073741824.0)*10.0).round()/10.0;
            let mem_percent = (((mem_used/mem_total)*100.0)*10.0).round()/10.0;
            //println!("mem%: {}", mem_percent);
            let up_time = convert_time(System::uptime());
            //println!("{}", up_time);
            // let usage_txt_clone = usage_txt.clone();
            // let mem_txt_clone = mem_txt.clone();
            let up_time_clone = up_time.clone();
            let handle_cpu_mem_clone = handle_cpu_mem.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_cpu_mem_clone.upgrade() {
                    window.set_cpuUsage(cpu_global);
                    window.set_memUsage(mem_used);
                    window.set_memPercent(mem_percent);
                    window.set_uptime(up_time_clone.into());
                }
            }).unwrap();

            thread::sleep(Duration::from_secs(1));
        }
    });


    // let mut networks = Networks::new_with_refreshed_list();
    // thread::sleep(Duration::from_millis(10));

    // networks.refresh(true);

    // for (interface_name, _) in &networks{
    //     println!("Interface name: {}", interface_name);
    // }


    let handle_net = handle.clone();
    std::thread::spawn(move || {
        let mut networks = Networks::new_with_refreshed_list();
        loop{
            networks.refresh(true);
            let received_before: u64 = networks
                .iter()
                .filter(|(name, _)| {
                    !name.to_lowercase().contains("npcap") && 
                    !name.to_lowercase().contains("loopback") && 
                    !name.to_lowercase().contains("virtual") &&
                    !name.to_lowercase().contains("vmware") &&
                    !name.to_lowercase().contains("vethernet")
                })
                .map(|(_, data)| data.received())
                .sum();
            let transmitted_before: u64 = networks
            .iter()
            .filter(|(name, _)| {
                    !name.to_lowercase().contains("npcap") && 
                    !name.to_lowercase().contains("loopback") && 
                    !name.to_lowercase().contains("virtual") &&
                    !name.to_lowercase().contains("vmware") &&
                    !name.to_lowercase().contains("vethernet")
                })
            .map(|(_, data)| data.transmitted())
            .sum();


            thread::sleep(Duration::from_secs(1));


            networks.refresh(true);
            let received_after: u64 = networks
                .iter()
                .filter(|(name, _)| {
                    !name.to_lowercase().contains("npcap") && 
                    !name.to_lowercase().contains("loopback") && 
                    !name.to_lowercase().contains("virtual") &&
                    !name.to_lowercase().contains("vmware") &&
                    !name.to_lowercase().contains("vethernet")
                })
                .map(|(_, data)| data.received())
                .sum();
            let transmitted_after: u64 = networks
                .iter()
                .filter(|(name, _)| {
                    !name.to_lowercase().contains("npcap") && 
                    !name.to_lowercase().contains("loopback") && 
                    !name.to_lowercase().contains("virtual") &&
                    !name.to_lowercase().contains("vmware") &&
                    !name.to_lowercase().contains("vethernet")
                })
                .map(|(_, data)| data.transmitted())
                .sum();

            let down_speed = received_after.saturating_sub(received_before);
            let up_speed = transmitted_after.saturating_sub(transmitted_before);

            let down_mbps = (((down_speed as f64 * 8.0)/ 1000000.0)*10.0).round()/10.0;
            let up_mbps = (((up_speed as f64 * 8.0)/ 1000000.0)*10.0).round()/10.0;

            //println!("Download: {:.2}Mbps | Upload: {:.2}Mbps",down_mbps, up_mbps);

            let handle_net_clone = handle_net.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_net_clone.upgrade(){
                    window.set_netDownload(down_mbps as f32);
                    window.set_netUpload(up_mbps as f32);
                }
            }).unwrap();
        }
    });


    let handle_disk = handle.clone();
    thread::spawn(move || {
        let mut disks = Disks::new_with_refreshed_list();
        loop{
            disks.refresh(true);
            let read_before = disks.list()[0].usage().read_bytes;
            let write_before = disks.list()[0].usage().written_bytes;

            thread::sleep(Duration::from_secs(1));

            disks.refresh(true);
            let read_after = disks.list()[0].usage().read_bytes;
            let write_after = disks.list()[0].usage().written_bytes;

            let read_speed = (((read_after-read_before) as f32 / 1048576.0)*10.0).round()/10.0;
            let write_speed = (((write_after-write_before) as f32 / (1048576.0))*10.0).round()/10.0;

            let handle_disk_clone = handle_disk.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_disk_clone.upgrade() {
                    window.set_readSpeed(read_speed);
                    window.set_writeSpeed(write_speed);
                }
            }).unwrap();

        }
    });

    main_window.run()?;
    Ok(())

}
