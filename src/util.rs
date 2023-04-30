
pub fn is_rpi() -> bool {
    if cfg!(target_arch="arm") && 
    cfg!(target_os="linux") &&
    cfg!(target_env="gnu")
    {
        true
    }else{
        false
    }
}