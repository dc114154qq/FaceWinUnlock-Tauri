fn main(){
    let stra = String::from("tauriUserName::hello world");
    if stra.contains("tauriUserName::"){
        println!("{}",stra.replace("tauriUserName::",""));
    }
}