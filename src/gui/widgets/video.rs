use anyhow::Error;
use druid::{
	kurbo::Circle,
	piet::{ImageFormat, InterpolationMode},
	widget::{Controller, FillStrat, Image},
	BoxConstraints, Color, Env, Event, EventCtx, ExtEventSink, ImageBuf, LayoutCtx, LifeCycle,
	LifeCycleCtx, MouseButton, PaintCtx, RenderContext, Selector, SingleUse, Size, Target,
	UpdateCtx, Widget,
};
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use num_traits::ToPrimitive;

use crate::{
	gui::{
		controller::cmd,
		data::video::{
			Position, VideoError, VideoError::Duration, VideoPlayer, VideoPlayerState, VideoView,
			VideoViewState,
		},
	},
	media::thumbnail::Thumbnail,
};

impl VideoView {
	/// Create new camera view
	pub fn new() -> Self {
		let image_buf = ImageBuf::default();
		let image = Image::new(image_buf)
			.fill_mode(FillStrat::Fill)
			.interpolation_mode(InterpolationMode::Bilinear);

		Self { image, player: None, event: None }
	}
}

impl Widget<VideoViewState> for VideoView {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut VideoViewState, env: &Env) {
		if let Event::Command(command) = event {
			if let Some(image_buf) = command.get(cmd::VIDEO_FRAME) {
				if self.player.is_some() {
					let player = self.player.as_ref().unwrap();
					for msg in player.bus.iter() {
						match msg.view() {
							gst::MessageView::Error(err) => panic!("{:#?}", err),
							gst::MessageView::Eos(_) => {
								println!("EOS");
							}
							_ => {}
						}
					}
					if data.percentage == data.pre_percentage {
						let position = player.position().as_secs();
						let percentage = position as f64 / data.duration as f64;
						data.position = position;
						data.percentage = percentage;
						data.pre_percentage = percentage;
					}
					if data.position == data.duration{
						ctx.submit_command(cmd::PLAY_PAUSE)
					}
				}
				self.image.set_image_data(image_buf.to_owned());
				ctx.request_paint();
			}
			if let Some(_) = command.get(cmd::PLAY_PAUSE) {
				if let Some(ref player) = self.player {
					player.pipeline.set_state(gst::State::Paused);
					data.state = VideoPlayerState::Paused;
				}
				// ctx.request_paint();
			}
			if let Some(_) = command.get(cmd::PLAY_RESUME) {
				if let Some(ref player) = self.player {
					player.pipeline.set_state(gst::State::Playing);
					data.state = VideoPlayerState::Playing;
				}
			}
			if let Some(duration) = command.get(cmd::PLAYBACK_DURATION) {
				data.duration = *duration;
			}
			if let Some(_) = command.get(cmd::PLAY) {
				if let Some(ref player) = self.player {
					player.pipeline.set_state(gst::State::Playing);
				}
				// ctx.request_paint();
			}
			if let Some(position) = command.get(cmd::PLAY_SEEK) {
				if let Some(ref player) = self.player {
					if data.state != VideoPlayerState::Playing {
						player.set_muted(true);
						player.pipeline.set_state(gst::State::Playing);
						player
							.pipeline
							.seek_simple(
								gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
								*position * gst::ClockTime::SECOND,
							)
							.unwrap();
						player.pipeline.set_property("mute", false);
						player.pipeline.state(gst::ClockTime::from_mseconds(20)).0.unwrap();
						player.pipeline.set_state(gst::State::Paused);
					} else {
						player.set_muted(true);
						player
							.pipeline
							.seek_simple(
								gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
								*position * gst::ClockTime::SECOND,
							)
							.unwrap();
						player.pipeline.set_property("mute", false);
					}
					data.position = *position;
				}
			}
		}

		self.image.event(ctx, event, data, env)
	}

	fn lifecycle(
		&mut self,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		data: &VideoViewState,
		env: &Env,
	) {
		match event {
			LifeCycle::WidgetAdded => {
				println!("uri {:?}", std::path::PathBuf::from(file!()).parent().unwrap());
				let uri = &url::Url::from_file_path(
					std::path::PathBuf::from(file!())
						.parent()
						.unwrap()
						.join("../../../.media/test.mp4")
						.canonicalize()
						.unwrap(),
				)
				.unwrap();
				/*				let png_data = ImageBuf::from_data(include_bytes!("../../../.media/PicWithAlpha.png")).unwrap();

				let  image = Image::new(png_data)			.fill_mode(FillStrat::Contain)
					.interpolation_mode(InterpolationMode::Bilinear);*/
				let thumbnail = Thumbnail::new(uri.as_str(), 7).unwrap();
				let image_buf = thumbnail.receiver.recv().unwrap();

				let player = VideoPlayer::new(uri, false, ctx.get_external_handle()).unwrap();
				self.player = Some(player);
				self.image.set_image_data(image_buf.to_owned());
				ctx.get_external_handle()
					.submit_command(cmd::PLAYBACK_DURATION, thumbnail.duration, Target::Auto)
					.map_err(|e| gstreamer::FlowError::Error)
					.unwrap();
			}

			_ => {}
		}
		self.image.lifecycle(ctx, event, data, env)
	}

	fn update(
		&mut self,
		ctx: &mut UpdateCtx,
		old_data: &VideoViewState,
		data: &VideoViewState,
		env: &Env,
	) {
		/*		if old_data.image_dimensions != data.image_dimensions {
			ctx.request_layout();
		}*/
		//TODO
		self.image.update(ctx, old_data, data, env)
	}

	fn layout(
		&mut self,
		ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		data: &VideoViewState,
		env: &Env,
	) -> Size {
		self.image.layout(ctx, bc, data, env)
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &VideoViewState, env: &Env) {
		self.image.paint(ctx, data, env);
	}
}
pub struct VideoViewController {}

impl<W: Widget<VideoViewState>> Controller<VideoViewState, W> for VideoViewController {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &Event,
		data: &mut VideoViewState,
		env: &Env,
	) {
		match event {
			Event::MouseDown(mouse) => {
				if mouse.button == MouseButton::Left {
					if data.state == VideoPlayerState::Playing {
						ctx.submit_command(cmd::PLAY_PAUSE)
					} else {
						ctx.submit_command(cmd::PLAY_RESUME)
					}
					// ctx.set_active(true);
				}
			}
			_ => {}
		}
		child.event(ctx, event, data, env)
	}
}

impl Drop for VideoPlayer {
	fn drop(&mut self) {
		self.pipeline.set_state(gst::State::Null).expect("failed to set state");
	}
}

impl VideoPlayer {
	/// Create a new video player from a given video which loads from `uri`.
	///
	/// If `live` is set then no duration is queried (as this will result in an
	/// error and is non-sensical for live streams). Set `live` if the streaming
	/// source is indefinite (e.g. a live stream). Note that this will cause the
	/// duration to be zero.
	pub fn new(uri: &url::Url, live: bool, event_sink: ExtEventSink) -> Result<Self, VideoError> {
		// Initialize GStreamer
		gst::init()?;

		// Build the pipeline
		let uri =
			"https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
		let uri = url::Url::from_file_path(
			std::path::PathBuf::from(file!())
				.parent()
				.unwrap()
				.join("../../../.media/test.mp4")
				.canonicalize()
				.unwrap(),
		)
		.unwrap();
		println!("{:?}", uri);
		let pipeline = gst::ElementFactory::make("playbin", None).unwrap();
		pipeline.set_property("uri", uri.as_str());
		/// ************************** audio
		/// ****************************************
		// Create elements that go inside the sink bin
		let equalizer = gst::ElementFactory::make("equalizer-3bands", Some("equalizer"))
			.expect("Could not create equalizer element.");
		let convert = gst::ElementFactory::make("audioconvert", Some("convert"))
			.expect("Could not create audioconvert element.");
		let sink = gst::ElementFactory::make("autoaudiosink", Some("audio_sink"))
			.expect("Could not create autoaudiosink element.");

		// Create the sink bin, add the elements and link them
		let bin = gst::Bin::new(Some("audio_sink_bin"));
		bin.add_many(&[&equalizer, &convert, &sink]).unwrap();
		gst::Element::link_many(&[&equalizer, &convert, &sink]).expect("Failed to link elements.");

		let pad = equalizer.static_pad("sink").expect("Failed to get a static pad from equalizer.");
		let ghost_pad = gst::GhostPad::with_target(Some("sink"), &pad).unwrap();
		ghost_pad.set_active(true)?;
		bin.add_pad(&ghost_pad)?;

		// Configure the equalizer
		equalizer.set_property("band1", -24.0);
		equalizer.set_property("band2", -24.0);

		pipeline.set_property("audio-sink", &bin);

		/// ************************** video
		/// ****************************************
		// Create elements that go inside the sink bin
		let queue = gst::ElementFactory::make("queue", None).expect("Could not create queue element.");
		let convert = gst::ElementFactory::make("videoconvert", None)
			.expect("Could not create videoconvert element.");
		let scale = gst::ElementFactory::make("videoscale", None)
			.expect("Could not create videoscale element.");
		// let sink = gst::ElementFactory::make("autovideosink", None)
		// 	.map_err(|_| MissingElement("autovideosink"))?;
		let sink = gst::ElementFactory::make("appsink", None)
			.expect("Could not create autovideosink element.");

		// Create the sink bin, add the elements and link them
		let bin = gst::Bin::new(Some("video_sink_bin"));
		bin.add_many(&[&queue, &convert, &scale, &sink]).unwrap();
		gst::Element::link_many(&[&queue, &convert, &scale, &sink])
			.expect("Failed to link elements.");

		let pad = queue.static_pad("sink").expect("Failed to get a static pad from equalizer.");
		let ghost_pad = gst::GhostPad::with_target(Some("sink"), &pad).unwrap();
		ghost_pad.set_active(true)?;
		bin.add_pad(&ghost_pad)?;

		let video_sink = sink
			.dynamic_cast::<gst_app::AppSink>()
			.expect("Sink element is expected to be an appsink!");
		// caps=video/x-raw,format=BGRA,pixel-aspect-ratio=1/1
		video_sink.set_caps(Some(&gst::Caps::new_simple(
			"video/x-raw",
			&[("format", &"RGBA"), ("pixel-aspect-ratio", &gst::Fraction::from((1, 1)))],
		)));
		video_sink.set_callbacks(
			gst_app::AppSinkCallbacks::builder()
				.new_sample(move |sink| {
					let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
					let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
					let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;

					let pad = sink.static_pad("sink").ok_or(gst::FlowError::Error)?;

					let caps = pad.current_caps().ok_or(gst::FlowError::Error)?;
					let s = caps.structure(0).ok_or(gst::FlowError::Error)?;
					let width = s.get::<i32>("width").map_err(|_| gst::FlowError::Error)?;
					let height = s.get::<i32>("height").map_err(|_| gst::FlowError::Error)?;
					// println!("W: {:?}, H: {:?}", width, height);
					// Send original and processed image.
					let image = ImageBuf::from_raw(
						map.as_slice().to_owned(),
						ImageFormat::RgbaSeparate,
						width as _,
						height as _,
					);
					event_sink
						.submit_command(cmd::VIDEO_FRAME, image, Target::Auto)
						.map_err(|e| gstreamer::FlowError::Error)?;
					// let position = 		std::time::Duration::from_nanos(
					// 	pipeline.query_position::<gst::ClockTime>().map_or(0, |pos| pos.nseconds()),
					// ).as_secs();
					// event_sink
					// 	.submit_command(cmd::PLAYBACK_PROGRESS, position, Target::Auto)
					// 	.map_err(|e| gstreamer::FlowError::Error)?;

					Ok(gst::FlowSuccess::Ok)
				})
				.build(),
		);

		// Configure the equalizer
		equalizer.set_property("band1", -24.0);
		equalizer.set_property("band2", -24.0);

		pipeline.set_property("video-sink", &bin);
		pipeline.set_property("volume", 0.9);

		pipeline.set_state(gst::State::Paused)?;

		// wait for up to 5 seconds until the decoder gets the source capabilities
		pipeline.state(gst::ClockTime::from_seconds(5)).0?;

		// extract resolution and framerate
		// TODO(jazzfool): maybe we want to extract some other information too?
		let caps = pad.current_caps().ok_or(VideoError::Caps)?;
		let s = caps.structure(0).ok_or(VideoError::Caps)?;
		let width = s.get::<i32>("width").map_err(|_| VideoError::Caps)?;
		let height = s.get::<i32>("height").map_err(|_| VideoError::Caps)?;
		let framerate = s.get::<gst::Fraction>("framerate").map_err(|_| VideoError::Caps)?;

		/*		let duration = if !live {
			std::time::Duration::from_nanos(
				pipeline.query_duration::<gst::ClockTime>().ok_or(VideoError::Duration)?.nseconds(),
			)
		} else {
			std::time::Duration::from_secs(0)
		};*/

		Ok(VideoPlayer {
			bus: pipeline.bus().unwrap(),
			pipeline,

			volume: 0.0,
			width,
			height,
			framerate: num_rational::Rational32::new(
				framerate.numer() ,
				framerate.denom() ,
			)
				.to_f64().unwrap(/* if the video framerate is bad then it would've been implicitly caught far earlier */),
			duration: std::time::Duration::from_secs(0),

			paused: false,
			muted: false,
			looping: false,
			is_eos: false,
			restart_stream: false,
		})
	}

	/// Get the size/resolution of the video as `(width, height)`.
	#[inline(always)]
	pub fn size(&self) -> (i32, i32) {
		(self.width, self.height)
	}

	/// Set the volume multiplier of the audio.
	/// `0.0` = 0% volume, `1.0` = 100% volume.
	///
	/// This uses a linear scale, for example `0.5` is perceived as half as
	/// loud.
	pub fn set_volume(&mut self, volume: f64) {
		self.pipeline.set_property("volume", &volume);
	}

	/// Set if the audio is muted or not, without changing the volume.
	pub fn set_muted(&self, muted: bool) {
		// self.muted = muted;
		self.pipeline.set_property("mute", &muted);
	}

	/// Get if the stream ended or not.
	#[inline(always)]
	pub fn eos(&self) -> bool {
		self.is_eos
	}

	/// Set if the media will loop or not.
	#[inline(always)]
	pub fn set_looping(&mut self, looping: bool) {
		self.looping = looping;
	}

	/// Set if the media is paused or not.
	pub fn set_paused(&mut self, paused: bool) {
		self.pipeline
			.set_state(if paused {
				gst::State::Paused
			} else {
				gst::State::Playing
			})
			.unwrap(/* state was changed in ctor; state errors caught there */);
		self.paused = paused;

		// Set restart_stream flag to make the stream restart on the next
		// Message::NextFrame
		if self.is_eos && !paused {
			self.restart_stream = true;
		}
	}

	/*	/// Get if the media is paused or not.
	#[inline(always)]
	pub fn paused(&self) -> bool {
		self.paused
	}*/

	/// Jumps to a specific position in the media.
	/// The seeking is not perfectly accurate.
	pub fn seek(&mut self, position: impl Into<Position>) -> Result<(), Error> {
		self.pipeline.seek_simple(gst::SeekFlags::FLUSH, position.into())?;
		Ok(())
	}
	pub fn position(&self) -> std::time::Duration {
		std::time::Duration::from_nanos(
			self.pipeline.query_position::<gst::ClockTime>().map_or(0, |pos| pos.nseconds()),
		)
		.into()
	}
	/*
	/// Get the current playback position in time.
	pub fn position(&self) -> std::time::Duration {
		std::time::Duration::from_nanos(
			self.pipeline.query_position::<gst::ClockTime>().map_or(0, |pos| pos.nseconds()),
		)
		.into()
	}

	/// Get the media duration.
	#[inline(always)]
	pub fn duration(&self) -> std::time::Duration {
		self.duration
	}*/

	/*	/// Generates a list of thumbnails based on a set of positions in the media.
	///
	/// Slow; only needs to be called once for each instance.
	/// It's best to call this at the very start of playback, otherwise the
	/// position may shift.
	pub fn thumbnails(&mut self, positions: &[Position]) -> Result<Vec<img::Handle>, VideoError> {
		let paused = self.paused();
		let pos = self.position();
		self.set_paused(false);
		let out = positions
			.iter()
			.map(|&pos| {
				self.seek(pos)?;
				self.wait.recv().map_err(|_| VideoError::Sync)?;
				Ok(self.frame_image())
			})
			.collect();
		self.set_paused(paused);
		self.seek(pos)?;
		out
	}*/

	/*	/// Restarts a stream; seeks to the first frame and unpauses, sets the `eos`
	/// flag to false.
	pub fn restart_stream(&mut self) -> Result<(), VideoError> {
		self.is_eos = false;
		self.set_paused(false);
		self.seek(0)?;
		Ok(())
	}*/
}
