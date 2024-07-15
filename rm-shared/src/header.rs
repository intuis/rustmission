use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Header {
    Name,
    SizeWhenDone,
    Progress,
    Eta,
    DownloadRate,
    UploadRate,
    DownloadDir,
    Padding,
    UploadRatio,
    UploadedEver,
    Id,
    ActivityDate,
    AddedDate,
    PeersConnected,
    SmallStatus,
}

impl Header {
    pub fn default_constraint(&self) -> Constraint {
        match self {
            Self::Name => Constraint::Max(70),
            Self::SizeWhenDone => Constraint::Length(12),
            Self::Progress => Constraint::Length(12),
            Self::Eta => Constraint::Length(12),
            Self::DownloadRate => Constraint::Length(12),
            Self::UploadRate => Constraint::Length(12),
            Self::DownloadDir => Constraint::Max(70),
            Self::Padding => Constraint::Length(2),
            Self::UploadRatio => Constraint::Length(6),
            Self::UploadedEver => Constraint::Length(12),
            Self::Id => Constraint::Length(4),
            Self::ActivityDate => Constraint::Length(14),
            Self::AddedDate => Constraint::Length(12),
            Self::PeersConnected => Constraint::Length(6),
            Self::SmallStatus => Constraint::Length(1),
        }
    }

    pub fn header_name(&self) -> &'static str {
        match *self {
            Self::Name => "Name",
            Self::SizeWhenDone => "Size",
            Self::Progress => "Progress",
            Self::Eta => "ETA",
            Self::DownloadRate => "Download",
            Self::UploadRate => "Upload",
            Self::DownloadDir => "Directory",
            Self::Padding => "",
            Self::UploadRatio => "Ratio",
            Self::UploadedEver => "Up Ever",
            Self::Id => "Id",
            Self::ActivityDate => "Last active",
            Self::AddedDate => "Added",
            Self::PeersConnected => "Peers",
            Self::SmallStatus => "",
        }
    }
}
