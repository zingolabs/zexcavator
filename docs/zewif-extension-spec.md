# ZExCavator's ZeWIF Extension Specification (WIP)

> ⚠️ This specification is a draft and is subject to change.

```
{
    "<attachment_data>"
} [
    'conformsTo': "https://github.com/zingolabs/zexcavator/blob/main/docs/zewif-extension-spec.md",
    'vendor': "org.zingolabs"
]
```

where `<attachment_data>` is an envelope with the following structure:

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

Here, \<seed\> an object containing a binary seed from `zcashd`. For more information, see https://github.com/zcash/zcash/issues/5573#issue-1145986602.
