// Import needed modules
use anyhow::Result;
use rseip::client::ab_eip::*; 
use rseip::precludes::*;

// Create async main fucntion to communicate with Ethernet/IP server
#[tokio::main]
pub async fn main() -> Result<()> {
    // Variable declaration
    let host = "127.0.0.1:53998";
    let mut client = AbEipClient::new_host_lookup(host).await?;
    // Loop to gather arrayed tag
    println!("[+] Looping through flag");
    let mut flag: String = "".to_string();
    for i in 0..50 {
        let tag_name = format!("FLAG[{}]",i);
        let path = EPath::parse_tag(&tag_name)?;
        let res: TagValue<i16> = client.read_tag(path).await?;
        let val = res.value;
        if val == 0 {
            break;
        }
        let letter: char = (val as u8) as char;
        flag.push(letter);
    }

    println!("Flag: {}",flag);

    Ok(())
}