// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, error::Error, time::Duration};
use slint::ComponentHandle;
use sysinfo::{CpuRefreshKind, Disks, Networks, RefreshKind, System};

mod timeConvert;
use timeConvert::convert_time;



slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {

    let mut sys = System::new_all();
    sys.refresh_all();
    let disks = Disks::new_with_refreshed_list();
    let dk_usage = disks.list()[0].usage();
    println!("{:?}", dk_usage);


    let os_name= System::name().unwrap_or("default".to_string());
    let os_ver = System::os_version().unwrap_or("default".to_string());
    let cpu_brand = sys.cpus()[0].brand();
    let core_count = System::physical_core_count().unwrap();
    let mut thread_count = 0;
    for i in sys.cpus(){
        thread_count += 1;
    }

    println!("Name: {}", os_name);
    println!("Brand: {}",cpu_brand);
    println!("Core: {}", core_count);
    println!("Thread: {}", thread_count);

    let cpu_count = format!("{core_count}({thread_count})");
    println!("Both: {}", cpu_count);

    


    let main_window = MainWindow::new()?;
    main_window.set_osName(os_name.into());
    main_window.set_osVersion(os_ver.into());
    main_window.set_cpuName(cpu_brand.into());
    main_window.set_cpuCount(cpu_count.into());

    let handle = main_window.as_weak();

    let handle_cpu_mem = handle.clone();
    std::thread::spawn(move || {
        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let cpu_global = sys.global_cpu_usage();
            let mem_global = (sys.used_memory()) as f32/(1024.0*1024.0*1024.0);
            let usage_txt = (cpu_global*10.0).round()/10.0;
            let mem_txt = (mem_global*10.0).round()/10.0;
            let up_time = convert_time(System::uptime());
            //println!("{}", up_time);

            let usage_txt_clone = usage_txt.clone();
            let mem_txt_clone = mem_txt.clone();
            let up_time_clone = up_time.clone();
            let handle_cpu_mem_clone = handle_cpu_mem.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_cpu_mem_clone.upgrade() {
                    window.set_cpuUsage(usage_txt_clone);
                    window.set_memUsage(mem_txt_clone);
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

            println!("Download: {:.2}Mbps | Upload: {:.2}Mbps",down_mbps, up_mbps);

            let handle_net_clone = handle_net.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_net_clone.upgrade(){
                    window.set_netDownload(down_mbps as f32);
                    window.set_netUpload(up_mbps as f32);
                }
            }).unwrap();

            //thread::sleep(Duration::from_secs(1));

        }
    });

    main_window.run()?;
    Ok(())

}
