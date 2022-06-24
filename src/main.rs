use anyhow::Result;
use druid::{AppLauncher, LocalizedString, WindowDesc};
use druid_video::{
	gui,
	gui::data::{
		video::{VideoPlayerState, VideoViewState},
		Theme,
	},
};
use gui::{data::AppState, ui::root_widget};

fn main() -> Result<()> {
	let window = WindowDesc::new(root_widget())
		.title(LocalizedString::new("Window-Title").with_placeholder("druid video"))
		.window_size((640.0, 480.0));
	let launcher = AppLauncher::with_window(window);
	let state = AppState {
		video: VideoViewState {
			state: VideoPlayerState::Paused,
			current_item: "/home/damo/rust/yosef/druid_video/.media/test.mp4".to_string(),
			duration: Default::default(),
			position: Default::default(),
			percentage: 0.0,
			pre_percentage: 0.0,
			seeking_enabled: true,
		},
		theme: Theme::Light,
	};

	launcher.log_to_console().launch(state).expect("running app");
	Ok(())
}
