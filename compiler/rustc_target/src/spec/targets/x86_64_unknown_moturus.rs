use crate::spec::{base, CodeModel, Target};

pub fn target() -> Target {
    let mut base = base::moturus::opts();
    base.cpu = "x86-64".into();
    base.max_atomic_width = Some(64);
    base.code_model = Some(CodeModel::Small);

    Target {
        llvm_target: "x86_64-unknown-none".into(),
        metadata: crate::spec::TargetMetadata {
            description: None,
            tier: None,
            host_tools: None,
            std: None,
        },
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
        arch: "x86_64".into(),
        options: base,
    }
}
