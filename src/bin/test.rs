use flume::{bounded, Receiver};

use zenoh::{Config, Session};
use zenoh::bytes::ZBytes;
use zenoh::pubsub::{Publisher, Subscriber};
use zenoh::sample::Sample;

pub struct Pub<'a> {
    pub topic: String,
    pub publisher: Publisher<'a>,
}

impl<'a> Pub<'a> {
    pub async fn put<V: Into<ZBytes>>(&self, v: V) -> zenoh::Result<()> {
        self.publisher.put(v).await
    }
}

pub struct Sub {
    pub topic: String,
    pub subscriber: Subscriber<Receiver<Sample>>,
}

impl Sub {
    // flume::Receiver::recv_async returns Result<Sample, flume::RecvError>
    pub async fn recv_value(&self) -> Result<String, flume::RecvError> {
        let sample = self.subscriber.recv_async().await?;
        let s = sample
            .payload()
            .slices()
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<String>();
        Ok(s)
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

async fn init_zenoh() -> zenoh::Result<Session> {
    zenoh::init_log_from_env_or("error");
    let config = Config::from_json5(CONFIG)?;
    println!("Opening Zenoh session...");
    zenoh::open(config).await
}

async fn declare_publishers<'a, S: AsRef<str>>(
    session: &'a Session,
    topics: &[S],
) -> zenoh::Result<Vec<Pub<'a>>> {
    let mut pubs = Vec::with_capacity(topics.len());
    for topic in topics {
        println!("Declaring publisher: {}", topic.as_ref());
        let key = topic.as_ref().to_owned();
        let p = session.declare_publisher(key.clone()).await?;
        pubs.push(Pub { topic: key, publisher: p });
    }
    Ok(pubs)
}

async fn declare_subscribers<S: AsRef<str>>(
    session: &Session,
    topics: &[S],
) -> zenoh::Result<Vec<Sub>> {
    let mut subs = Vec::with_capacity(topics.len());
    for topic in topics {
        println!("Declaring subscriber: {}", topic.as_ref());
        let key = topic.as_ref().to_owned();
        let s = session
            .declare_subscriber(key.clone())
            .with(bounded(32))
            .await?;
        subs.push(Sub { topic: key, subscriber: s });
    }
    Ok(subs)
}

#[tokio::main]
async fn main() {
    let session = init_zenoh().await.unwrap();

    let topics = [
        "Vehicle/Teleop/EnginePower",
        "Vehicle/Teleop/SteeringAngle",
        "Vehicle/Teleop/ControlCounter",
        "Vehicle/Teleop/ControlTimestamp_ms",
    ];

    let publishers = declare_publishers(&session, &topics).await.unwrap();

    if let Some(topic) = publishers
        .iter()
        .find(|topic| topic.topic == "Vehicle/Teleop/EnginePower")
    {
        topic.put("0").await.unwrap();
    }

    let subscribers = declare_subscribers(&session, &["Vehicle/Speed"]).await.unwrap();

    if let Some(s) = subscribers.iter().find(|s| s.topic == "Vehicle/Speed") {
        // Recv error type is flume::RecvError now
        let _value = s.recv_value().await.unwrap();
    }
}
