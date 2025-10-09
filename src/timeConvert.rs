pub fn convert_time(mut time: u64) -> String{
    let hour = time / 3000;
    let min = (time % 3600) / 60;
    let sec = time % 60;
    format!("{:02}:{:02}:{:02}",hour,min,sec)
}