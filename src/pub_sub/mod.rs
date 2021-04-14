use std::{
    sync::{
        Arc,
        Mutex // TODO: RWLock
    },
    collections::{
        HashMap
    },
    hash::{
        Hash
    },
    ops::{
        Deref,
        DerefMut
    },
    fmt::{
        Debug
    }
};
use tracing::{
    instrument,
    debug,
    trace
};
use tokio::{
    sync::{
        mpsc::{
            Sender,
            Receiver,
            channel
        }
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Subscription<K, V>
where 
    K: Hash + Eq + Debug,
    V: Debug
{
    subscribers: Arc<Mutex<HashMap<K, Sender<V>>>>,
    key: K,
    receiver: Receiver<V>
}

impl<K, V> Deref for Subscription<K, V> 
where 
    K: Hash + Eq + Debug,
    V: Debug
{
    type Target = Receiver<V>;

    #[instrument(level = "trace", skip(self))]
    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl<K, V> DerefMut for Subscription<K, V> 
where 
    K: Hash + Eq + Debug,
    V: Debug
{
    #[instrument(level = "trace", skip(self))]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}

impl<K, V> Drop for Subscription<K, V> 
where 
    K: Hash + Eq + Debug,
    V: Debug
{
    #[instrument(level = "trace", skip(self))]
    fn drop(&mut self) {
        let mut sub_lock = self.subscribers.lock().expect("Mutex lock failed");
        sub_lock.remove(&self.key);
        drop(sub_lock);
        
        trace!("Subscriber removed for key: {:?}", self.key);
    }
}

impl<K, V> Subscription<K, V> 
where 
    K: Hash + Eq + Debug,
    V: Debug
{
    pub fn get_key(&self) -> &K{
        &self.key
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

pub struct PubSub<K, V>
where 
    K: Hash + Eq + Clone,
    V: Debug
{
    subscribers: Arc<Mutex<HashMap<K, Sender<V>>>>
}

impl<K, V> Default for PubSub<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Debug
{
    fn default() -> Self {
        PubSub{
            subscribers: Default::default()
        }
    }
}

impl<K,V> PubSub<K, V>
where 
    K: Hash + Eq + Clone + Debug,
    V: Debug
{
    pub fn new() -> PubSub<K, V>{
        Default::default()
    }
    
    #[instrument(level = "trace", skip(self, receiver_init_func))]
    pub fn subscribe_if_does_not_exist<F>(&self, k: K, buffer: usize, receiver_init_func: F) -> Sender<V>
    where
        F: FnOnce(Subscription<K, V>)
    {
        let mut subscribers_lock = self.subscribers.lock().expect("Mutex lock failed");

        if let Some(sender) = subscribers_lock.get(&k) {
            sender.clone()
        }else{
            let (tx, rx) = channel::<V>(buffer);
            subscribers_lock.insert(k.clone(), tx.clone());
            drop(subscribers_lock);

            trace!("New subscription for: {:?}", k);

            let sub = Subscription{
                key: k,
                subscribers: self.subscribers.clone(),
                receiver: rx
            };
            receiver_init_func(sub);

            tx
        }
    }
}