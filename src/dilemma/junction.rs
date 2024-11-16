use bevy::{prelude::*, sprite::Anchor, text::BreakLineOn};
use crate::dilemma::Dilemma;
use crate::{
    train::{
		TrainBundle, 
		STEAM_TRAIN
	},
	track::TrackBundle,
    lever::{
		OPTION_1_COLOR, 
		OPTION_2_COLOR,
	},
    person::{
		PERSON,
		PersonSprite,
		BounceAnimation,
		EmoticonSprite
	},
	motion::PointToPointTranslation
};

