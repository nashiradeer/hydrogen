//! # Hydrogen Player // Utils
//!
//! Utilities to be used when implementing a new engine/backend.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use async_trait::async_trait;
use rand::{thread_rng, Rng};

use crate::{QueueAdd, Result, Track};

/// Allows initialization or fetch of a [`Track`], used by some utilities such as [`Queue`] to provide standard backend APIs completely ready to use.
#[async_trait]
pub trait ToTrack {
    /// Initializes or fetches a new [`Track`] with this track's data.
    async fn track(&self) -> Result<Track>;
}

/// Standard in-memory queue system, can and should be used by any backend that does not have its own queue system.
#[derive(Clone)]
pub struct Queue<T: ToTrack> {
    /// The queue stored in memory.
    queue: Arc<RwLock<Vec<T>>>,

    /// Determines whether the index will not be updated.
    repeat_music: Arc<AtomicBool>,

    /// Determines whether the index should be updated randomly.
    random_next: Arc<AtomicBool>,

    /// Determines whether the index should return to zero once it reaches the queue limit.
    cyclic_queue: Arc<AtomicBool>,

    /// Determines whether the track should play as soon as the index is updated.
    autoplay: Arc<AtomicBool>,

    /// The index of the current track.
    index: Arc<RwLock<usize>>,

    /// How many items can be placed in this queue.
    max_size: usize,
}

impl<T: ToTrack + Clone> Queue<T> {
    /// Initializes a new queue controller.
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            repeat_music: Arc::new(AtomicBool::new(false)),
            random_next: Arc::new(AtomicBool::new(false)),
            cyclic_queue: Arc::new(AtomicBool::new(false)),
            autoplay: Arc::new(AtomicBool::new(true)),
            index: Arc::new(RwLock::new(0)),
            max_size,
        }
    }

    /// The maximum amount of items that can fit in this queue.
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// The length of the queue.
    pub fn len(&self) -> usize {
        self.queue.read().unwrap().len()
    }

    /// The total capacity of the queue.
    pub fn capacity(&self) -> usize {
        self.queue.read().unwrap().capacity()
    }

    /// Gets true if the current song will repeat.
    pub fn repeat_music(&self) -> bool {
        self.repeat_music.load(Ordering::Relaxed)
    }

    /// Gets true if the next song will be randomly chosen.
    pub fn random_next(&self) -> bool {
        self.random_next.load(Ordering::Relaxed)
    }

    /// Gets true if the queue will go back to the beginning instead of hanging on the last song.
    pub fn cyclic_queue(&self) -> bool {
        self.cyclic_queue.load(Ordering::Relaxed)
    }

    /// Gets true if the next song will be played automatically.
    pub fn autoplay(&self) -> bool {
        self.autoplay.load(Ordering::Relaxed)
    }

    /// Sets if the current song should be repeated.
    pub fn set_repeat_music(&self, repeat_music: bool) {
        self.repeat_music.store(repeat_music, Ordering::Relaxed);
    }

    /// Sets if the next song should be random.
    pub fn set_random_next(&self, random_next: bool) {
        self.random_next.store(random_next, Ordering::Relaxed);
    }

    /// Sets if the queue should start over as soon as it reaches the end.
    pub fn set_cyclic_queue(&self, cyclic_queue: bool) {
        self.cyclic_queue.store(cyclic_queue, Ordering::Relaxed);
    }

    /// Sets if the next song should be played automatically.
    pub fn set_autoplay(&self, autoplay: bool) {
        self.autoplay.store(autoplay, Ordering::Relaxed);
    }

    /// Updates the index, returning the track that will be played now or `None` if index is out of bounds.
    pub fn set_index(&self, new_index: usize) -> Option<T> {
        // A ReadGuard to the queue.
        let queue = self.queue.read().unwrap();

        // A WriteGuard to the index.
        let mut index = self.index.write().unwrap();

        // Get the track from the queue.
        let track = queue.get(new_index);

        // Check if new_index is valid before update the index.
        if track.is_some() {
            *index = new_index;
        }

        track.cloned()
    }

    /// Updates the index, returning the track that will be played now or `None` if autoplay is disabled.
    pub fn next(&self) -> Option<T> {
        // A ReadGuard to the queue.
        let queue = self.queue.read().unwrap();

        // This variable allows the index updater to disable the autoplay.
        let mut disable_autoplay = false;

        // A WriteGuard to the index.
        let mut index = self.index.write().unwrap();

        // Check if index need to be updated.
        if !self.repeat_music.load(Ordering::Relaxed) {
            // Check if index will not be random.
            if self.random_next.load(Ordering::Relaxed) {
                // Updates the index, incrementing it.
                *index += 1;

                // Check if the index has exceeded the queue length.
                if index.ge(&queue.len()) {
                    // Check if the queue is in cyclic mode.
                    if self.cyclic_queue.load(Ordering::Relaxed) {
                        // Reset index to the start of the queue.
                        *index = 0;
                    } else {
                        // Keeps the index in the end of the queue.
                        *index = queue.len() - 1;

                        // Disable autoplay to avoid repeat the last track.
                        disable_autoplay = true;
                    }
                }
            } else {
                // Generates a random index.
                *index = rand::thread_rng().gen_range(0..queue.len());
            }
        }

        // Check if autoplay is enabled.
        if !disable_autoplay && self.autoplay.load(Ordering::Relaxed) {
            return queue.get(index.clone()).cloned();
        }

        None
    }

    /// Add new tracks to the queue.
    fn add(&self, mut songs: Vec<T>) -> QueueAdd {
        // A WriteGuard to the queue.
        let mut queue = self.queue.write().unwrap();

        // If the queue already full, skips this operation.
        if queue.len() >= self.max_size {
            return QueueAdd {
                offset: 0,
                track: Vec::new(),
                truncated: true,
            };
        }

        // Check if queue has space enough for the new songs, truncating the vector with the new songs if necessary.
        let mut truncated = false;
        if queue.len() + songs.len() > self.max_size {
            truncated = true;
            songs.truncate(self.max_size - (queue.len() + songs.len()));
        }

        // The offset at which the songs for this operation start.
        let offset = queue.len();

        // Collect the tracks for the result before extending the queue.
        let tracks = songs.iter().map(|i| i.track()).collect();

        // Extends the queue.
        queue.extend(songs);

        QueueAdd {
            track: tracks,
            offset,
            truncated,
        }
    }

    /// It uses iterators to capture a part of the queue and generate a `Vec<Track>`. Should be used to implement the `queue` function of the `Backend` trait.
    pub async fn queue(&self, offset: usize, size: usize) -> Result<Vec<Track>> {
        // Prepare for reading the queue.
        let queue = self.queue.read().unwrap();

        // Allocate a vector for the tracks.
        let mut track_queue = Vec::with_capacity(size);

        // It goes through the queue converting or fetching the tracks.
        for item in queue.iter().skip(offset).take(size) {
            track_queue.push(item.track().await?);
        }

        // Deallocate unused space.
        track_queue.shrink_to_fit();

        Ok(track_queue)
    }

    /// Shuffles the queue, changing the tracks position.
    pub fn shuffle(&self) -> Option<T> {
        // A WriteGuard to the index, the index need be updated with the new position.
        let mut index = self.index.write().unwrap();

        // A WriteGuard to the queue.
        let mut queue = self.queue.write().unwrap();

        // Get the current track to be searched when the new index is needed to be know.
        let current_track = queue.get(index).map(|i| i.track());

        // A new vector that will substitute the old queue.
        let mut new_queue = Vec::with_capacity(queue.capacity());

        // Executes while the current queue is not cleared.
        while queue.len() > 0 {
            // Generates a random index to be used to fetch a random a track from the current queue.
            let i = thread_rng().gen_range(0..queue.len());

            // Removes 'i' from the current queue to be insert in the new queue.
            let track = queue.swap_remove(index);

            // Adds the track to the new queue.
            new_queue.push(track);
        }

        // Updates the index, searching for the new position of the track in the new queue or setting to 0 if not found.
        if let Some(current_track) = current_track {
            if let Some(new_index) = new_queue.iter().position(|i| i.track() == current_track) {
                *index = new_index;

                // Will not be need to replace the current track playing.
                return None;
            }
        }
    }
}
