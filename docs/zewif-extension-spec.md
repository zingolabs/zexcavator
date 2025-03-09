# ZExCavator's ZeWIF Extension Specification (WIP)

> ⚠️ This specification is a draft and is subject to change.

```
{
    "<attachment_data>"
} [
    'conformsTo': "https://github.com/zingolabs/zexcavator/blob/33c6e476f79093ec3ff976ab8f25b8cbd5ee6f67/docs/zewif-extension-spec.md",
    'vendor': "org.zingolabs"
]
```

where `<attachment_data>` is an envelope with the following format:

```
{
    "wallet": [
        "hasSeed": "<seed>" [
            "generates": "<emergency_recovery_phrase>"
        ]
        'version': "<semver_version>"
    ]
}
```
