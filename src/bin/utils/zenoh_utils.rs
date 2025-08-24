use flume::{bounded, Receiver};

use zenoh::{Config, Session};
use zenoh::bytes::ZBytes;
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::sample::Sample;

#[derive(Debug)]
pub struct Pub<'a> {
    pub topic: String,
    pub publisher: Publisher<'a>,
}

impl<'a> Pub<'a> {
    /// Publish while logging what we send (bytes + UTF-8 view).
    pub async fn put<V: Into<ZBytes>>(&self, v: V) -> zenoh::Result<()> {
        // Convert once so we can both log and send the same payload.
        let payload: ZBytes = v.into();

        // Raw bytes (flattened)
        let bytes: Vec<u8> = payload
            .slices()
            .fold(Vec::new(), |mut b, x| { b.extend_from_slice(x); b });

        // UTF-8 view (lossy to avoid panics)
        let as_string = payload
            .slices()
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<String>();

        println!(
            "→ [PUT] '{}' | {} bytes\n   bytes: {:?}\n   text : \"{}\"",
            self.topic,
            bytes.len(),
            bytes,
            as_string
        );

        self.publisher.put(payload).await
    }
}

#[derive(Debug)]
pub struct Sub {
    pub topic: String,
    pub subscriber: Subscriber<Receiver<Sample>>,
}

impl Sub {
    /// Receive one payload as a String, logging details like your earlier sample-based code.
    /// flume::Receiver::recv_async returns Result<Sample, flume::RecvError>
    pub async fn recv_value(&self) -> Result<String, flume::RecvError> {
        let sample = self.subscriber.recv_async().await?;

        // Flatten bytes for logging (mirrors your first example)
        let bytes: Vec<u8> = sample
            .payload()
            .slices()
            .fold(Vec::new(), |mut b, x| { b.extend_from_slice(x); b });

        // Lossy UTF-8 view so non-UTF payloads don’t panic
        let text = sample
            .payload()
            .slices()
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<String>();

        println!(
            "← [{}] '{}' | {} bytes\n   bytes: {:?}\n   text : \"{}\"",
            sample.kind(),
            sample.key_expr().as_str(),
            bytes.len(),
            bytes,
            text
        );

        Ok(text)
    }
}

const CONFIG: &str = r#"{
    "mode": "client",
    "connect": {
        "endpoints": ["tcp/zenoh:7447"],
        "timeout_ms": -1,
        "exit_on_failure": false
    }
}"#;

pub async fn init_zenoh() -> zenoh::Result<Session> {
    zenoh::init_log_from_env_or("error");
    let config = Config::from_json5(CONFIG)?;
    println!("Opening Zenoh session...");
    zenoh::open(config).await
}

pub async fn declare_publishers<'a, S: AsRef<str>>(
    session: &'a Session,
    topics: &[S],
) -> zenoh::Result<Vec<Pub<'a>>> {
    let mut pubs = Vec::with_capacity(topics.len());
    for topic in topics {
        let key = topic.as_ref().to_owned();
        println!("Declaring publisher: {}", key);
        let p = session.declare_publisher(key.clone()).await?;
        pubs.push(Pub { topic: key, publisher: p });
    }
    Ok(pubs)
}

pub async fn declare_subscribers<S: AsRef<str>>(
    session: &Session,
    topics: &[S],
) -> zenoh::Result<Vec<Sub>> {
    let mut subs = Vec::with_capacity(topics.len());
    for topic in topics {
        let key = topic.as_ref().to_owned();
        println!("Declaring subscriber: {}", key);
        let s = session
            .declare_subscriber(key.clone())
            .with(bounded(32))
            .await?;
        subs.push(Sub { topic: key, subscriber: s });
    }
    Ok(subs)
}
// 
// #[tokio::main]
// async fn main() {
//     let session = init_zenoh().await.unwrap();
// 
//     let topics = [
//         "Vehicle/Teleop/EnginePower",
//         "Vehicle/Teleop/SteeringAngle",
//         "Vehicle/Teleop/ControlCounter",
//         "Vehicle/Teleop/ControlTimestamp_ms",
//     ];
// 
//     let publishers = declare_publishers(&session, &topics).await.unwrap();
// 
//     if let Some(topic) = publishers
//         .iter()
//         .find(|topic| topic.topic == "Vehicle/Teleop/EnginePower")
//     {
//         
//         topic.put("10").await.unwrap();
//     }
// 
//     let subscribers = declare_subscribers(&session, &["Vehicle/Speed"]).await.unwrap();
// 
//     if let Some(s) = subscribers.iter().find(|s| s.topic == "Vehicle/Speed") {
//         let _value = s.recv_value().await.unwrap();
//     }
// }
