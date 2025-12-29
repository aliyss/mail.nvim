use super::autocomplete::Commands;

pub fn command_tree() -> Commands {
    Commands::from_iter([
        (
            "account",
            Commands::from_iter([
                ("add", Commands::default()),
                (
                    "default",
                    Commands::from_iter([("set", Commands::default())]),
                ),
                ("edit", Commands::default()),
                ("list", Commands::default()),
                ("remove", Commands::default()),
            ]),
        ),
        (
            "config",
            Commands::from_iter([
                ("file", Commands::default()),
                ("location", Commands::default()),
                (
                    "himalaya",
                    Commands::from_iter([
                        ("file", Commands::default()),
                        (
                            "file-location",
                            Commands::from_iter([
                                ("reset", Commands::default()),
                                ("set", Commands::default()),
                            ]),
                        ),
                    ]),
                ),
                (
                    "email-view",
                    Commands::from_iter([(
                        "as-command",
                        Commands::from_iter([("set", Commands::default())]),
                    )]),
                ),
                (
                    "userhandholding",
                    Commands::from_iter([(
                        "set",
                        Commands::from_iter([
                            ("true", Commands::default()),
                            ("false", Commands::default()),
                        ]),
                    )]),
                ),
            ]),
        ),
        (
            "help",
            Commands::from_iter([
                ("keybindings", Commands::default()),
                ("about", Commands::default()),
                ("changelog", Commands::default()),
                ("license", Commands::default()),
                ("contribute", Commands::default()),
                ("support", Commands::default()),
                ("issue-report", Commands::default()),
                ("feature-request", Commands::default()),
            ]),
        ),
        (
            "email",
            Commands::from_iter([
                ("copy", Commands::default()),
                ("create", Commands::default()),
                ("delete", Commands::default()),
                ("discard", Commands::default()),
                ("download-attachments", Commands::default()),
                ("export", Commands::default()),
                (
                    "flag",
                    Commands::from_iter([
                        ("add", Commands::default()),
                        ("clear", Commands::default()),
                        ("remove", Commands::default()),
                    ]),
                ),
                ("forward", Commands::default()),
                ("list", Commands::default()),
                ("move", Commands::default()),
                ("reply", Commands::default()),
                ("reply-all", Commands::default()),
                ("send", Commands::default()),
                ("save-as-draft", Commands::default()),
                ("save-as-template", Commands::default()),
                ("toggle-read", Commands::default()),
                (
                    "thread",
                    Commands::from_iter([
                        ("copy", Commands::default()),
                        ("download-attachments", Commands::default()),
                        ("export", Commands::default()),
                        ("list", Commands::default()),
                        ("mark-read", Commands::default()),
                        ("move", Commands::default()),
                        ("next", Commands::default()),
                        ("previous", Commands::default()),
                    ]),
                ),
                ("view-as", Commands::default()),
            ]),
        ),
        (
            "folder",
            Commands::from_iter([
                ("create", Commands::default()),
                (
                    "default",
                    Commands::from_iter([
                        (
                            "draft",
                            Commands::from_iter([
                                ("reset", Commands::default()),
                                ("set", Commands::default()),
                            ]),
                        ),
                        (
                            "inbox",
                            Commands::from_iter([
                                ("reset", Commands::default()),
                                ("set", Commands::default()),
                            ]),
                        ),
                        (
                            "trash",
                            Commands::from_iter([
                                ("reset", Commands::default()),
                                ("set", Commands::default()),
                            ]),
                        ),
                    ]),
                ),
                ("delete", Commands::default()),
                ("expunge", Commands::default()),
                ("list", Commands::default()),
                ("purge", Commands::default()),
                ("rename", Commands::default()),
            ]),
        ),
        (
            "tag",
            Commands::from_iter([
                ("create", Commands::default()),
                ("delete", Commands::default()),
                ("edit", Commands::default()),
                ("list", Commands::default()),
                ("save", Commands::default()),
            ]),
        ),
        (
            "template",
            Commands::from_iter([
                ("create", Commands::default()),
                ("delete", Commands::default()),
                ("edit", Commands::default()),
                ("list", Commands::default()),
                ("save", Commands::default()),
                (
                    "default",
                    Commands::from_iter([("set", Commands::default())]),
                ),
                ("type", Commands::from_iter([("set", Commands::default())])),
            ]),
        ),
        (
            "ui",
            Commands::from_iter([
                ("close", Commands::default()),
                ("refresh", Commands::default()),
                ("toggle", Commands::default()),
                (
                    "view",
                    Commands::from_iter([
                        (
                            "component",
                            Commands::from_iter([
                                ("config-file", Commands::default()),
                                (
                                    "feature",
                                    Commands::from_iter([("set", Commands::default())]),
                                ),
                                ("list", Commands::default()),
                                (
                                    "tag",
                                    Commands::from_iter([
                                        ("add", Commands::default()),
                                        ("remove", Commands::default()),
                                        ("clear", Commands::default()),
                                    ]),
                                ),
                                ("toggle", Commands::default()),
                                ("type", Commands::from_iter([("set", Commands::default())])),
                            ]),
                        ),
                        ("config-file", Commands::default()),
                        (
                            "default",
                            Commands::from_iter([
                                ("set", Commands::default()),
                                ("clear", Commands::default()),
                            ]),
                        ),
                        ("delete", Commands::default()),
                        ("list", Commands::default()),
                        ("reset", Commands::default()),
                        ("save", Commands::default()),
                    ]),
                ),
            ]),
        ),
    ])
}
