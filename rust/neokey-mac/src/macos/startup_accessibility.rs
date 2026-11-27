#[derive(Debug, PartialEq, Eq)]
pub(super) enum Action {
    Wait,
    InstallTap,
    InstallTapAndRecommendRestart,
    Relaunch,
}

pub(super) enum StartupAccessibility {
    Ready,
    AwaitingGrant,
    RelaunchAttempted,
}

impl StartupAccessibility {
    pub(super) fn new(trusted_at_launch: bool) -> Self {
        if trusted_at_launch {
            Self::Ready
        } else {
            Self::AwaitingGrant
        }
    }

    pub(super) fn action(&mut self, trusted: bool, bundled: bool) -> Action {
        if !trusted {
            return Action::Wait;
        }
        match self {
            Self::Ready => Action::InstallTap,
            Self::AwaitingGrant if bundled => {
                *self = Self::RelaunchAttempted;
                Action::Relaunch
            }
            Self::AwaitingGrant => {
                *self = Self::Ready;
                Action::InstallTapAndRecommendRestart
            }
            Self::RelaunchAttempted => Action::Wait,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_app_relaunches_once_when_trust_is_granted() {
        let mut startup = StartupAccessibility::new(false);
        assert_eq!(startup.action(false, true), Action::Wait);
        assert_eq!(startup.action(true, true), Action::Relaunch);
        assert_eq!(startup.action(true, true), Action::Wait);
        assert_eq!(startup.action(false, true), Action::Wait);
        assert_eq!(startup.action(true, true), Action::Wait);
    }

    #[test]
    fn app_trusted_at_launch_can_always_install_its_tap_when_trusted() {
        let mut startup = StartupAccessibility::new(true);
        assert_eq!(startup.action(true, true), Action::InstallTap);
        assert_eq!(startup.action(false, true), Action::Wait);
        assert_eq!(startup.action(true, true), Action::InstallTap);
    }

    #[test]
    fn bare_binary_installs_its_tap_and_recommends_restart_once() {
        let mut startup = StartupAccessibility::new(false);
        assert_eq!(startup.action(false, false), Action::Wait);
        assert_eq!(
            startup.action(true, false),
            Action::InstallTapAndRecommendRestart
        );
        assert_eq!(startup.action(true, false), Action::InstallTap);
    }
}
