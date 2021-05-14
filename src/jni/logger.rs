use jni::JNIEnv;
use jni::objects::{JClass, JString};
use log::Level;

fn handle_log(env: JNIEnv, message: JString, level: Level) {
    let message = env.get_string(message);
    match message {
        Ok(message) => {
            let message: String = message.into();
            log::log!(target: "titan-rs", level, "{}", message);
        }
        Err(err) => log::error!(target: "titan-rs", "{:?}", err)
    }
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Logger_error(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Error)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Logger_warn(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Warn)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Logger_info(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Info)
}

#[allow(non_snake_case)]
#[no_mangle]
extern "system" fn Java_native_Logger_debug(env: JNIEnv, _class: JClass, message: JString) {
    handle_log(env, message, Level::Debug)
}
