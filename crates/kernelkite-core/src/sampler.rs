//! Sampling abstraction.
//!
//! A [`Sampler`] produces one [`Frame`] per call to [`Sampler::sample`]. The
//! core ships three backends:
//!
