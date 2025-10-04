// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, error::Error, time::Duration};
use slint::ComponentHandle;
use sysinfo::System;



slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {


    let mut sys = System::new_all();
    sys.refresh_all();
    let sys_name= System::name().unwrap_or("default".to_string());
    let sys_ver = System::os_version().unwrap_or("default".to_string());
    println!("Name: {}", sys_name);


    let main_window = MainWindow::new()?;
    main_window.set_sysName(sys_name.into());
    main_window.set_sysVer(sys_ver.into());

    let handle = main_window.as_weak();

    std::thread::spawn(move || {
        loop {
            sys.refresh_all();
            let cpu_global = sys.global_cpu_usage();
            let mem_global = (sys.used_memory()) as f64/(1024.0*1024.0*1024.0);
            let usage_txt = format!("{:.2}%", cpu_global);
            let mem_txt = format!("{:.2}GB", mem_global);
            //println!("{},  {}", usage_txt, mem_txt);

            let usage_txt_clone = usage_txt.clone();
            let mem_txt_clone = mem_txt.clone();
            let handle_clone = handle.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = handle_clone.upgrade() {
                    window.set_cpuUsage(usage_txt_clone.clone().into());
                    window.set_memUsage(mem_txt_clone.clone().into());
                }
            }).unwrap();

            thread::sleep(Duration::from_secs(1));
        }
    });

    main_window.run()?;
    Ok(())

}
