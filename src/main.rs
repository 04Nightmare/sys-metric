// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, collections::HashMap, error::Error, os::windows::process, rc::Rc, thread, time::Duration};
use slint::{ComponentHandle, SharedString, VecModel, ModelRc};
use sysinfo::{CpuRefreshKind, Disks, Motherboard, Networks, Pid, ProcessRefreshKind, RefreshKind, System};
use gfxinfo::active_gpu;

mod timeConvert;
use timeConvert::convert_time;


slint::include_modules!();


fn main() -> Result<(), slint::PlatformError> {

    let main_window = MainWindow::new()?;
    let handle = main_window.as_weak();



    main_window.on_referesh_stats({
        let mut refresh = System::new();
        move || {
            refresh.refresh_all();
            thread::sleep(Duration::from_secs(3));
        }
    });



    let handle_gpu = handle.clone();
    thread::spawn(move || {
        let gpu = active_gpu().expect("No GPU found");
            let gpu_model = gpu.model().to_string();
            let gpu_vram = (gpu.info().total_vram())/1073741824;
            let handle_gpu_clone = handle_gpu.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_gpu_clone.upgrade(){
                    window.set_gpu(gpu_model.into());
                    window.set_vram(gpu_vram as i32);
                }
            }).unwrap();
    });

    thread::sleep(Duration::from_secs(2));


    let mut sys = System::new_all();
    sys.refresh_all();

    
    // use std::sync::{Arc, Mutex};

    // main_window.on_referesh_stats({
    //     let refresh = Arc::new(Mutex::new(System::new()));
    //     move || {
    //         let refresh = refresh.clone();
    //         thread::spawn(move || {
    //             if let Ok(mut sys) = refresh.lock() {
    //                 sys.refresh_all();
    //             }
    //             thread::sleep(Duration::from_secs(5));
    //         });
    //     }
    // });

    

    let board_info = if let Some(m) = Motherboard::new() {
        let info = format!("{} ({})", m.vendor_name().unwrap_or("N/A".to_string()), m.name().unwrap_or("N/A".to_string()));
        info
    } else {
        String::from("N/A")
    };


    let os_name= System::name().unwrap_or("default".to_string());
    let os_ver = System::os_version().unwrap_or("default".to_string());
    let cpu_brand = sys.cpus()[0].brand();
    let core_count = System::physical_core_count().unwrap();
    let thread_count = sys.cpus().len();
    let total_ram = sys.total_memory().div_ceil(1073741824);
    let total_swap = sys.total_swap().div_ceil(1073741824);
    let ram_and_swap = format!("{}GB (Virtual-{}GB)", total_ram, total_swap);
    let cpu_count = format!("{core_count}({thread_count})");


    main_window.set_osName(os_name.into());
    main_window.set_osVersion(os_ver.into());
    main_window.set_cpuName(cpu_brand.into());
    main_window.set_cpuCount(cpu_count.into());
    main_window.set_motherBoard(board_info.into());
    main_window.set_memory(ram_and_swap.into());

    

    let handle_cpu_mem = handle.clone();
    std::thread::spawn(move || {
        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();

            let cpu_global = (sys.global_cpu_usage()*10.0).round()/10.0;
            let mem_used = ((sys.used_memory() as f32/1073741824.0)*10.0).round()/10.0;
            let mem_total = ((sys.total_memory() as f32/1073741824.0)*10.0).round()/10.0;
            let mem_percent = (((mem_used/mem_total)*100.0)*10.0).round()/10.0;
            let up_time = convert_time(System::uptime());
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

    let handle_process = handle.clone();
    thread::spawn(move || {
        let mut prs = System::new();
        loop{
            prs.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, true, ProcessRefreshKind::nothing().with_cpu().with_memory().without_tasks());
            let mut totals: HashMap<String, u64> = HashMap::new();
            for (_, proc_) in prs.processes() {
                let name = proc_.name().to_string_lossy().to_string();
                let mem = proc_.memory();
                *totals.entry(name).or_insert(0) += mem;
            }

            let mut process_totals: Vec<(String, u64)> = totals.into_iter().collect();
            
            process_totals.sort_by(|a, b| b.1.cmp(&a.1));

            let top_prs: Vec<_> = process_totals.into_iter().take(8).collect();

            let process_data: Vec<ProcessItemData> = top_prs.iter()
                .map(|(name, mem)| ProcessItemData {
                    name: SharedString::from(name.clone()),
                    mem: SharedString::from(format!("{:.1} MB", *mem as f64 / (1024.0*1024.0))),
                })
                .collect();

            let handle_process_clone = handle_process.clone();
            slint::invoke_from_event_loop(move || {
                let process_model = VecModel::from(process_data);
                if let Some(window) = handle_process_clone.upgrade(){
                    window.set_process_model(ModelRc::from(Rc::new(process_model)));
                }
            }).unwrap();

            thread::sleep(Duration::from_secs(2));
        }
    });

    main_window.run()?;
    Ok(())

}
