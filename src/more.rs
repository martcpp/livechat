use anyhow::Ok;
use tokio::time::sleep;
use std::time::Duration;


async fn counter_to(num:u32) {

    for count in 1..=num {
       sleep(Duration::from_millis(100)).await;
        println!("{}",count);
      }


}





#[tokio::main]  

async fn main() -> anyhow::Result<()>{
  let couter_to_10 = counter_to(10);
  tokio::pin!(couter_to_10);

  tokio::select! {
    _ = counter_to(5) => {
      println!("Counter 3 finished")
    },
    _ = &mut couter_to_10 => { 
      println!("Counter 5 finished")
    },
  };
  println!("stop counting");
    println!("jk, keep counting");
    couter_to_10.await; 
    println!("finished counting to 10");

    Ok(())
}