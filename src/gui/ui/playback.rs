use std::time::Duration;

use druid::{
	widget::{Controller, Either, Flex, KnobStyle, Label, SizedBox, Slider, ViewSwitcher},
	Color, Cursor, Data, Env, Event, EventCtx, KeyOrValue, MouseButton, PaintCtx, Point, Rect,
	RenderContext, Size, Widget, WidgetExt,
};

use crate::gui::{
	controller::cmd,
	data::{
		video::{VideoPlayer, VideoPlayerState, VideoViewState},
		AppState,
	},
	widgets::{
		empty::Empty,
		icons::{self, SvgIcon},
		theme,
	},
};

pub fn panel_widget() -> impl Widget<AppState> {
	// let seek_bar = Either::new(|playback, _| playback.current_item.is_some(),
	// SeekBar::new(), Empty);

	let controls = Either::new(
		|video: &VideoViewState, _| !video.current_item.is_empty(),
		player_widget(),
		Empty,
	);
	Flex::column()
		.with_child(Either::new(
			|state: &VideoViewState, _| !state.current_item.is_empty(),
			Slider::new()
				.with_range(0.0, 1.0)
				.track_color(KeyOrValue::Concrete(Color::RED))
				.with_step(0.01)
				.lens(VideoViewState::percentage)
				.expand_width()
				.boxed()
				.controller(SliderController {}),
			Empty,
		))
		.with_default_spacer()
		.with_child(controls)
		.lens(AppState::video)
	// .controller(PlaybackController::new())
}

fn player_widget() -> impl Widget<VideoViewState> {
	Flex::row()
		.with_child(player_play_pause_widget())
		.with_default_spacer()
		.with_child(Either::new(
			|state: &VideoViewState, _| !state.current_item.is_empty(),
			durations_widget(),
			Empty,
		))
		.padding(theme::grid(2.0))
}

fn player_play_pause_widget() -> impl Widget<VideoViewState> {
	ViewSwitcher::new(
		|video: &VideoViewState, _| video.state,
		|state, _, _| match state {
			VideoPlayerState::Playing => icons::PAUSE
				.scale((theme::grid(3.0), theme::grid(3.0)))
				.padding(theme::grid(1.0))
				.border(theme::GREY_500, 1.0)
				.on_click(|ctx, _, _| ctx.submit_command(cmd::PLAY_PAUSE))
				.boxed(),
			VideoPlayerState::Paused => icons::PLAY
				.scale((theme::grid(3.0), theme::grid(3.0)))
				.padding(theme::grid(1.0))
				.border(theme::GREY_500, 1.0)
				.on_click(|ctx, _, _| ctx.submit_command(cmd::PLAY_RESUME))
				.boxed(),
			VideoPlayerState::Stopped => Empty.boxed(),
		},
	)
}
/*
fn small_button_widget<T: Data>(svg: &SvgIcon) -> impl Widget<T> {
	svg.scale((theme::grid(2.0), theme::grid(2.0)))
		.padding(theme::grid(1.0))
		.rounded(theme::BUTTON_BORDER_RADIUS)
}

fn faded_button_widget<T: Data>(svg: &SvgIcon) -> impl Widget<T> {
	svg.scale((theme::grid(2.0), theme::grid(2.0)))
		.with_color(theme::PLACEHOLDER_COLOR)
		.padding(theme::grid(1.0))
		.rounded(theme::BUTTON_BORDER_RADIUS)
}*/

fn durations_widget() -> impl Widget<VideoViewState> {
	Label::dynamic(|state: &VideoViewState, _| {
		format!(
			"{} / {}",
			as_minutes_and_seconds(state.position),
			as_minutes_and_seconds(state.duration)
		)
	})
	.with_text_size(theme::TEXT_SIZE_SMALL)
	.with_text_color(theme::PLACEHOLDER_COLOR)
	.fix_width(theme::grid(8.0))
}

pub fn as_minutes_and_seconds(dur: u64) -> String {
	let minutes = dur / 60;
	let seconds = dur % 60;
	format!("{}∶{:02}", minutes, seconds)
}
/*
pub fn as_human(dur: Duration) -> String {
	HumanTime::from(dur).to_text_en(
		time_humanize::Accuracy::Rough,
		time_humanize::Tense::Present,
	)
}*/
/*struct SeekBar {
	loudness_path: BezPath,
}

impl SeekBar {
	fn new() -> Self {
		Self { loudness_path: BezPath::new() }
	}
}

impl Widget<NowPlaying> for SeekBar {
	fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut NowPlaying, _env: &Env) {
		match event {
			Event::MouseMove(_) => {
				ctx.set_cursor(&Cursor::Pointer);
			}
			Event::MouseDown(mouse) => {
				if mouse.button == MouseButton::Left {
					ctx.set_active(true);
				}
			}
			Event::MouseUp(mouse) => {
				if ctx.is_active() && mouse.button == MouseButton::Left {
					if ctx.is_hot() {
						let fraction = mouse.pos.x / ctx.size().width;
						ctx.submit_command(cmd::PLAY_SEEK.with(fraction));
					}
					ctx.set_active(false);
				}
			}
			_ => {}
		}
	}

	fn lifecycle(
		&mut self,
		ctx: &mut LifeCycleCtx,
		event: &LifeCycle,
		_data: &NowPlaying,
		_env: &Env,
	) {
		match &event {
			LifeCycle::Size(_bounds) => {
				// self.loudness_path = compute_loudness_path(bounds, &data);
			}
			LifeCycle::HotChanged(_) => {
				ctx.request_paint();
			}
			_ => {}
		}
	}

	fn update(
		&mut self,
		ctx: &mut UpdateCtx,
		old_data: &NowPlaying,
		data: &NowPlaying,
		_env: &Env,
	) {
		if !old_data.same(data) {
			ctx.request_paint();
		}
	}

	fn layout(
		&mut self,
		_ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		_data: &NowPlaying,
		_env: &Env,
	) -> Size {
		Size::new(bc.max().width, theme::grid(1.0))
	}

	fn paint(&mut self, ctx: &mut PaintCtx, data: &NowPlaying, env: &Env) {
		if self.loudness_path.is_empty() {
			paint_progress_bar(ctx, data, env)
		} else {
			paint_audio_analysis(ctx, data, &self.loudness_path, env)
		}
	}
}*/

/*fn paint_progress_bar(ctx: &mut PaintCtx, data: &NowPlaying, env: &Env) {
	let elapsed_time = data.progress.as_secs_f64();
	let total_time = data.item.duration().as_secs_f64();

	let (elapsed_color, remaining_color) = if ctx.is_hot() {
		(env.get(theme::GREY_200), env.get(theme::GREY_500))
	} else {
		(env.get(theme::GREY_300), env.get(theme::GREY_600))
	};
	let bounds = ctx.size();

	let elapsed_frac = elapsed_time / total_time;
	let elapsed_width = bounds.width * elapsed_frac;
	let remaining_width = bounds.width - elapsed_width;
	let elapsed = Size::new(elapsed_width, bounds.height).round();
	let remaining = Size::new(remaining_width, bounds.height).round();

	ctx.fill(&Rect::from_origin_size(Point::ORIGIN, elapsed), &elapsed_color);
	ctx.fill(&Rect::from_origin_size(Point::new(elapsed.width, 0.0), remaining), &remaining_color);
}*/

pub struct SliderController {}

impl<W: Widget<VideoViewState>> Controller<VideoViewState, W> for SliderController {
	fn event(
		&mut self,
		child: &mut W,
		ctx: &mut EventCtx,
		event: &Event,
		data: &mut VideoViewState,
		env: &Env,
	) {
		match event {
			Event::MouseMove(mouse) => {
				ctx.set_cursor(&Cursor::Pointer);
				if ctx.is_active() {
					let position = data.duration as f64 * data.percentage;
					data.pre_percentage = data.percentage;
					ctx.submit_command(cmd::PLAY_SEEK.with(position as u64));
					data.seeking_enabled = false;
				}
			}
			Event::MouseDown(mouse) => {
				if mouse.button == MouseButton::Left {
					ctx.set_active(true);
				}
			}
			Event::MouseUp(mouse) => {
				if ctx.is_active() && mouse.button == MouseButton::Left {
					if data.seeking_enabled {
						let position = data.duration as f64 * data.percentage;
						data.pre_percentage = data.percentage;
						ctx.submit_command(cmd::PLAY_SEEK.with(position as u64));
					}
					data.seeking_enabled = true;
					ctx.set_active(false);
				}
			}
			_ => {}
		}
		child.event(ctx, event, data, env)
	}
}
