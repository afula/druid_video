use druid::{widget::Image, Data, ExtEventError, ExtEventSink, Lens};
use gst::prelude::*;
use gstreamer as gst;
use gstreamer::query::Uri;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoError {
	#[error("{0}")]
	Glib(#[from] glib::Error),
	#[error("{0}")]
	Bool(#[from] glib::BoolError),
	#[error("failed to get the gstreamer bus")]
	Bus,
	#[error("{0}")]
	StateChange(#[from] gst::StateChangeError),
	#[error("failed to cast gstreamer element")]
	Cast,
	#[error("{0}")]
	Io(#[from] std::io::Error),
	#[error("invalid URI")]
	Uri,
	#[error("failed to get media capabilities")]
	Caps,
	#[error("failed to query media duration or position")]
	Duration,
	#[error("failed to sync with playback")]
	Sync,
	#[error("{0}")]
	ExtEventError(#[from] ExtEventError),

	#[error("{0}")]
	PadLinkError(#[from] gst::PadLinkError),
	#[error("{0}")]
	FlowError(#[from] gstreamer::FlowError),
	#[error("{0}")]
	Other(#[from] anyhow::Error),
}

/// `CameraView` widget
pub struct VideoView {
	pub image: Image,
	pub player: Option<VideoPlayer>,
	pub event: Option<ExtEventSink>,
	// pub state: VideoViewState,
}

/// Position in the media.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Position {
	/// Position based on time.
	///
	/// Not the most accurate format for videos.
	Time(std::time::Duration),
	/// Position based on nth frame.
	Frame(u64),
}

impl From<Position> for gst::GenericFormattedValue {
	fn from(pos: Position) -> Self {
		match pos {
			Position::Time(t) => gst::ClockTime::from_nseconds(t.as_nanos() as _).into(),
			Position::Frame(f) => gst::format::Default(f).into(),
		}
	}
}

impl From<std::time::Duration> for Position {
	fn from(t: std::time::Duration) -> Self {
		Position::Time(t)
	}
}

impl From<u64> for Position {
	fn from(f: u64) -> Self {
		Position::Frame(f)
	}
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, Data)]
pub enum VideoPlayerState {
	Playing,
	Paused,
	Stopped,
}
#[derive(Data, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum VideoRate {
	D2,
	D5,
	M,
	I2,
	I5,
	I20,
}
#[derive(Clone, Debug, Data, Lens)]
pub struct VideoViewState {
	pub state: VideoPlayerState,
	pub current_item: String,
	pub duration: u64,
	pub position: u64,
	pub percentage: f64,
	pub pre_percentage: f64,
	pub seeking_enabled: bool,
	pub rate: f64,
	pub rate_set: bool,
}

/// Video player which handles multimedia playback.
pub struct VideoPlayer {
	pub bus: gst::Bus,
	pub pipeline: gst::Element,

	pub volume: f64,
	pub width: i32,
	pub height: i32,
	pub framerate: f64,
	pub duration: std::time::Duration,
	pub paused: bool,
	pub muted: bool,
	pub looping: bool,
	pub is_eos: bool,
	pub restart_stream: bool,
}
