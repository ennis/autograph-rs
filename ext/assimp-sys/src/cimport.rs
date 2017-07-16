use std::os::raw::{c_char, c_float, c_int, c_uint, c_void};

use cfileio::*;
use importerdesc::*;
use postprocess::*;
use scene::*;
use types::*;

pub type AiLogStreamCallback = Option<unsafe extern "system" fn(*const c_char, *mut c_char)>;

#[repr(C)]
pub struct AiLogStream {
    pub callback: AiLogStreamCallback,
    pub user: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AiPropertyStore {
    pub sentinel: c_char,
}

pub type AiBool = c_int;
pub const AI_FALSE: AiBool = 0;
pub const AI_TRUE: AiBool = 1;

extern {
    pub fn aiImportFile(
        file: *const c_char,
        flags: AiPostProcessSteps) -> *const AiScene;

    pub fn aiImportFileEx(
        file: *const c_char,
        flags: AiPostProcessSteps,
        fs: *mut AiFileIO) -> *const AiScene;

    pub fn aiImportFileExWithProperties(
        file: *const c_char,
        flags: AiPostProcessSteps,
        fs: *mut AiFileIO,
        props: *const AiPropertyStore) -> *const AiScene;

    pub fn aiImportFileFromMemory(
        buffer: *const c_char,
        length: c_uint,
        flags: AiPostProcessSteps,
        hint: *const c_char) -> *const AiScene;

    pub fn aiImportFileFromMemoryWithProperties(
        buffer: *const c_char,
        length: c_uint,
        flags: AiPostProcessSteps,
        hint: *const c_char,
        props: *const AiPropertyStore) -> *const AiScene;

    pub fn aiApplyPostProcessing(
        scene: *const AiScene,
        flags: AiPostProcessSteps) -> *const AiScene;

    pub fn aiGetPredefinedLogStream(
        stream: AiDefaultLogStream,
        file: *const c_char) -> AiLogStream;

    pub fn aiAttachLogStream(
        stream: *const AiLogStream);

    pub fn aiEnableVerboseLogging(
        enable: AiBool);

    pub fn aiDetachLogStream(
        stream: *const AiLogStream) -> AiReturn;

    pub fn aiDetachAllLogStreams();

    pub fn aiReleaseImport(
        scene: *const AiScene);

    pub fn aiGetErrorString() -> *const c_char;

    pub fn aiIsExtensionSupported(
        extension: *const c_char) -> AiBool;

    pub fn aiGetExtensionList(
        out: *mut AiString);

    pub fn aiGetMemoryRequirements(
        scene: *const AiScene,
        info: *mut AiMemoryInfo);

    pub fn aiCreatePropertyStore() -> *mut AiPropertyStore;

    pub fn aiReleasePropertyStore(
        store: *mut AiPropertyStore);

    pub fn aiSetImportPropertyInteger(
        store: *mut AiPropertyStore,
        name: *const c_char,
        value: c_int);

    pub fn aiSetImportPropertyFloat(
        store: *mut AiPropertyStore,
        name: *const c_char,
        value: c_float);

    pub fn aiSetImportPropertyString(
        store: *mut AiPropertyStore,
        name: *const c_char,
        value: *const AiString);

    pub fn aiSetImportPropertyMatrix(
        store: *mut AiPropertyStore,
        name: *const c_char,
        value: *const AiMatrix4x4);

    pub fn aiCreateQuaternionFromMatrix(
        quaternion: *mut AiQuaternion,
        matrix: *const AiMatrix3x3);

    pub fn aiDecomposeMatrix(
        matrix: *const AiMatrix4x4,
        scaling: *mut AiVector3D,
        rotation: *mut AiQuaternion,
        position: *mut AiVector3D);

    pub fn aiTransposeMatrix4(
        matrix: *mut AiMatrix4x4);

    pub fn aiTransposeMatrix3(
        matrix: *mut AiMatrix3x3);

    pub fn aiTransformVecByMatrix3(
        vector: *mut AiVector3D,
        matrix: *const AiMatrix3x3);

    pub fn aiTransformVecByMatrix4(
        vector: *mut AiVector3D,
        matrix: *const AiMatrix4x4);

    pub fn aiMultiplyMatrix4(
        dst: *mut AiMatrix4x4,
        src: *const AiMatrix4x4);

    pub fn aiMultiplyMatrix3(
        dst: *mut AiMatrix3x3,
        src: *const AiMatrix3x3);

    pub fn aiIdentityMatrix3(
        matrix: *mut AiMatrix3x3);

    pub fn aiIdentityMatrix4(
        matrix: *mut AiMatrix4x4);

    pub fn aiGetImportFormatCount() -> usize;

    pub fn aiGetImportFormatDescription(
        index: usize) -> *const AiImporterDesc;
}
