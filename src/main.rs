// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, error::Error, time::Duration};
use slint::ComponentHandle;
use sysinfo::{Disks, System, RefreshKind, CpuRefreshKind};

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

    std::thread::spawn(move || {
        loop {
            sys.refresh_all();
            let cpu_global = sys.global_cpu_usage();
            let mem_global = (sys.used_memory()) as f32/(1024.0*1024.0*1024.0);
            let usage_txt = (cpu_global*10.0).round()/10.0;
            let mem_txt = (mem_global*10.0).round()/10.0;
            let up_time = convert_time(System::uptime());
            //println!("{}", up_time);

            let usage_txt_clone = usage_txt.clone();
            let mem_txt_clone = mem_txt.clone();
            let up_time_clone = up_time.clone();
            let handle_clone = handle.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_clone.upgrade() {
                    window.set_cpuUsage(usage_txt_clone);
                    window.set_memUsage(mem_txt_clone);
                    window.set_uptime(up_time_clone.into());
                }
            }).unwrap();

            thread::sleep(Duration::from_secs(1));
        }
    });

    main_window.run()?;
    Ok(())

}
