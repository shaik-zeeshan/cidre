use std::ffi::c_void;

use crate::{arc, cf, cm, define_cf_type, define_opts, os};

#[cfg(feature = "cv")]
use crate::cv;

#[cfg(feature = "cat")]
use crate::cat;
#[cfg(feature = "cat")]
use std::ptr::slice_from_raw_parts;

define_opts!(
    #[doc(alias = "CMSampleBufferFlag")]
    pub Flags(u32)
);

impl Flags {
    /// Make sure memory involved in audio buffer lists is 16-byte aligned
    #[doc(alias = "kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment")]
    pub const AUDIO_BUFFER_LIST_ASSURE_16_BYTE_ALIGNMENT: Self = Self(1 << 0);
}

pub type SampleBufMakeDataReadyCb =
    extern "C" fn(sbuf: &SampleBuf, make_data_ready_refcon: *const c_void);

#[doc(alias = "CMSampleTimingInfo")]
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct SampleTimingInfo {
    /// The duration of the sample. If a single struct applies to
    /// each of the samples, they all will have this duration
    pub duration: cm::Time,

    /// The time at which the sample will be presented. If a single
    /// struct applies to each of the samples, this is the presentationTime of the
    /// first sample. The presentationTime of subsequent samples will be derived by
    /// repeatedly adding the sample duration.
    pub pts: cm::Time,

    /// The time at which the sample will be decoded. If the samples
    /// are in presentation order (eg. audio samples, or video samples from a codec
    /// that doesn't support out-of-order samples), this can be set to kCMTimeInvalid.
    pub dts: cm::Time,
}

impl SampleTimingInfo {
    #[doc(alias = "kCMTimingInfoInvalid")]
    #[inline]
    pub fn invalid() -> Self {
        unsafe { kCMTimingInfoInvalid }
    }
}

define_cf_type!(
    #[doc(alias = "CMSampleBuffer")]
    SampleBuf(cm::AttachBearer)
);

impl AsRef<cm::Buf> for SampleBuf {
    fn as_ref(&self) -> &cm::Buf {
        unsafe { std::mem::transmute(self) }
    }
}

unsafe impl Send for SampleBuf {}

/// An object that contains zero or more media samples of a uniform media type
///
/// Sample buffers are Core Foundation objects that the system uses to move media
/// sample data through the media pipeline. An instance of cm::SampleBuf contains
/// zero or more compressed (or uncompressed) samples of a particular media type
/// and contains one of the following:
///
///   - A cm::BlockBuf of one or more media samples
///   - A cm::ImageBuf, a reference to the format description
///      for the stream of cm::SampleBuf-s, size and timing information
///      for each of the contained media samples, and both buffer-level
///      and sample-level attachments
///
/// A sample buffer can contain both sample-level and buffer-level attachments.
/// Each individual sample in a buffer may provide attachments that include
/// information such as timestamps and video frame dependencies.
///
/// It’s possible for a sample buffer to describe samples it doesn’t yet contain.
/// For example, some media services may have access to sample size, timing,
/// and format information before they read the data. Such services may create
/// sample buffers with that information and insert them into queues early,
/// and attach (or fill) the buffer of media data later, when it becomes ready.
/// Sample buffers have the concept of data-readiness, which means you can test,
/// set, and force them to become ready “now.” It’s also possible for a sample buffer
/// to contain nothing but a special buffer-level attachment that describes a media
/// stream event (for example, “discontinuity: drain and reset decoder before processing
/// the next cm::SampleBuffer”).
///
/// [CMSampleBuffer](https://developer.apple.com/documentation/coremedia/cmsamplebuffer?language=objc)
impl SampleBuf {
    /// Returns whether or not a cm::SampleBuf's data is ready.
    ///
    /// Example:
    /// ```
    /// use cidre::{cf, cm};
    ///
    /// let res = cm::SampleBuf::new(
    ///     None,
    ///     true,
    ///     None,
    /// );
    ///
    /// let buf = res.unwrap();
    /// assert!(buf.data_is_ready());
    /// ```
    #[doc(alias = "CMSampleBufferDataIsReady")]
    #[inline]
    pub fn data_is_ready(&self) -> bool {
        unsafe { CMSampleBufferDataIsReady(self) }
    }

    #[doc(alias = "CMSampleBufferSetDataReady")]
    #[inline]
    pub fn set_data_ready(&mut self) {
        unsafe { CMSampleBufferSetDataReady(self) }
    }

    /// ```
    /// use cidre::{cf, cm};
    ///
    /// let res = cm::SampleBuf::new(
    ///     None,
    ///     true,
    ///     None,
    /// );
    ///
    /// let buf = res.unwrap();
    /// buf.show();
    /// ```

    #[inline]
    pub fn new(
        data_buf: Option<&cm::BlockBuf>,
        data_ready: bool,
        format_description: Option<&cm::FormatDesc>,
    ) -> os::Result<arc::R<SampleBuf>> {
        unsafe {
            os::result_unchecked(|res| {
                Self::create_in(
                    None,
                    data_buf,
                    data_ready,
                    None,
                    std::ptr::null(),
                    format_description,
                    0,
                    0,
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    res,
                )
            })
        }
    }

    /// [CMSampleBufferCreate](https://developer.apple.com/documentation/coremedia/1489723-cmsamplebuffercreate?language=objc)
    #[doc(alias = "CMSampleBufferCreate")]
    pub unsafe fn create_in(
        allocator: Option<&cf::Allocator>,
        data_buffer: Option<&cm::BlockBuf>,
        data_ready: bool,
        make_data_ready_cb: Option<&SampleBufMakeDataReadyCb>,
        make_data_ready_refcon: *const c_void,
        format_description: Option<&cm::FormatDesc>,
        num_samples: cm::ItemCount,
        num_samples_timing_entries: cm::ItemCount,
        sample_timing_array: *const SampleTimingInfo,
        num_sample_size_entries: cm::ItemCount,
        sample_size_array: *const usize,
        sample_buffer_out: *mut Option<arc::R<SampleBuf>>,
    ) -> os::Result {
        unsafe {
            CMSampleBufferCreate(
                allocator,
                data_buffer,
                data_ready,
                make_data_ready_cb,
                make_data_ready_refcon,
                format_description,
                num_samples,
                num_samples_timing_entries,
                sample_timing_array,
                num_sample_size_entries,
                sample_size_array,
                sample_buffer_out,
            )
            .result()
        }
    }

    #[doc(alias = "CMSampleBufferCreateForImageBuffer")]
    #[cfg(feature = "cv")]
    pub fn with_image_buf_in(
        allocator: Option<&cf::Allocator>,
        image_buf: &cv::ImageBuf,
        data_ready: bool,
        make_data_ready_cb: Option<&SampleBufMakeDataReadyCb>,
        make_data_ready_refcon: *const c_void,
        format_description: &cm::FormatDesc,
        sample_timing: &SampleTimingInfo,
    ) -> os::Result<arc::R<SampleBuf>> {
        unsafe {
            os::result_unchecked(|res| {
                CMSampleBufferCreateForImageBuffer(
                    allocator,
                    image_buf,
                    data_ready,
                    make_data_ready_cb,
                    make_data_ready_refcon,
                    format_description,
                    sample_timing,
                    res,
                )
            })
        }
    }

    #[doc(alias = "CMSampleBufferCreateForImageBuffer")]
    #[cfg(feature = "cv")]
    pub fn with_image_buf(
        image_buffer: &cv::ImageBuf,
        data_ready: bool,
        make_data_ready_cb: Option<&SampleBufMakeDataReadyCb>,
        make_data_ready_refcon: *const c_void,
        format_description: &cm::FormatDesc,
        sample_timing: &SampleTimingInfo,
    ) -> os::Result<arc::R<SampleBuf>> {
        unsafe {
            os::result_unchecked(|res| {
                CMSampleBufferCreateForImageBuffer(
                    None,
                    image_buffer,
                    data_ready,
                    make_data_ready_cb,
                    make_data_ready_refcon,
                    format_description,
                    sample_timing,
                    res,
                )
            })
        }
    }

    #[doc(alias = "CMSampleBufferGetImageBuffer")]
    #[cfg(feature = "cv")]
    #[inline]
    pub fn image_buf(&self) -> Option<&cv::ImageBuf> {
        unsafe { CMSampleBufferGetImageBuffer(self) }
    }

    #[doc(alias = "CMSampleBufferGetImageBuffer")]
    #[cfg(feature = "cv")]
    #[inline]
    pub fn image_buf_mut(&mut self) -> Option<&mut cv::ImageBuf> {
        unsafe { std::mem::transmute(CMSampleBufferGetImageBuffer(self)) }
    }

    #[doc(alias = "CMSampleBufferGetDataBuffer")]
    #[inline]
    pub fn data_buf(&self) -> Option<&cm::BlockBuf> {
        unsafe { CMSampleBufferGetDataBuffer(self) }
    }

    #[doc(alias = "CMSampleBufferSetDataBuffer")]
    #[inline]
    pub fn set_data_buf(&mut self, val: &cm::BlockBuf) -> os::Result {
        unsafe { CMSampleBufferSetDataBuffer(self, val).result() }
    }

    /// Returns the output duration of a sample buffer.
    #[doc(alias = "CMSampleBufferGetDuration")]
    #[inline]
    pub fn duration(&self) -> cm::Time {
        unsafe { CMSampleBufferGetDuration(self) }
    }

    /// Returns the output duration of a sample buffer.
    #[doc(alias = "CMSampleBufferGetOutputDuration")]
    #[inline]
    pub fn output_duration(&self) -> cm::Time {
        unsafe { CMSampleBufferGetOutputDuration(self) }
    }

    /// Returns the presentation timestamp that’s the earliest numerically
    /// of all the samples in a sample buffer.
    #[doc(alias = "CMSampleBufferGetPresentationTimeStamp")]
    #[inline]
    pub fn pts(&self) -> cm::Time {
        unsafe { CMSampleBufferGetPresentationTimeStamp(self) }
    }

    /// Returns the decode timestamp that’s the earliest numerically
    /// of all the samples in a sample buffer.
    #[doc(alias = "CMSampleBufferGetDecodeTimeStamp")]
    #[inline]
    pub fn dts(&self) -> cm::Time {
        unsafe { CMSampleBufferGetDecodeTimeStamp(self) }
    }

    /// Returns the output presentation timestamp of the CMSampleBuffer.
    #[doc(alias = "CMSampleBufferGetOutputPresentationTimeStamp")]
    #[inline]
    pub fn output_pts(&self) -> cm::Time {
        unsafe { CMSampleBufferGetOutputPresentationTimeStamp(self) }
    }

    #[doc(alias = "CMSampleBufferGetOutputDecodeTimeStamp")]
    pub fn output_dts(&self) -> cm::Time {
        unsafe { CMSampleBufferGetOutputDecodeTimeStamp(self) }
    }

    #[doc(alias = "CMSampleBufferSetOutputPresentationTimeStamp")]
    #[inline]
    pub fn set_output_pts(&self, val: cm::Time) {
        unsafe { CMSampleBufferSetOutputPresentationTimeStamp(self, val) }
    }

    #[doc(alias = "CMSampleBufferGetSampleTimingInfo")]
    #[inline]
    pub fn timing_info(&self, sample_index: cm::ItemIndex) -> os::Result<cm::SampleTimingInfo> {
        unsafe { os::result_init(|res| CMSampleBufferGetSampleTimingInfo(self, sample_index, res)) }
    }

    /// Returns the size in bytes of a specified sample in a 'cm::SampleBuf'.
    ///
    /// Size in bytes of the specified sample in the 'cm::SampleBuf'.
    #[doc(alias = "CMSampleBufferGetSampleSize")]
    #[inline]
    pub fn sample_size(&self, sample_index: cm::ItemIndex) -> usize {
        unsafe { CMSampleBufferGetSampleSize(self, sample_index) }
    }

    /// Returns the total size in bytes of sample data in a 'cm::SampleBuf'.
    ///
    /// Total size in bytes of sample data in the cm::SampleBuffer.
    /// If there are no sample sizes in this 'cm::SampleBuf', a size of 0 will be returned.  
    #[doc(alias = "CMSampleBufferGetTotalSampleSize")]
    #[inline]
    pub fn total_sample_size(&self) -> usize {
        unsafe { CMSampleBufferGetTotalSampleSize(self) }
    }

    #[doc(alias = "CMSampleBufferGetFormatDescription")]
    #[inline]
    pub fn format_desc(&self) -> Option<&cm::FormatDesc> {
        unsafe { CMSampleBufferGetFormatDescription(self) }
    }

    /// Returns a reference to a cm::SampleBuf's immutable array of mutable sample attachments dictionaries (one dictionary
    /// per sample in the 'cm::SampleBuf')
    #[doc(alias = "CMSampleBufferGetSampleAttachmentsArray")]
    #[inline]
    pub fn attaches(
        &self,
        create_if_necessary: bool,
    ) -> Option<&cf::ArrayOf<cf::DictionaryOf<cf::String, cf::Plist>>> {
        unsafe {
            std::mem::transmute(CMSampleBufferGetSampleAttachmentsArray(
                self,
                create_if_necessary,
            ))
        }
    }

    #[doc(alias = "CMSampleBufferGetSampleAttachmentsArray")]
    #[inline]
    pub fn attaches_mut(
        &mut self,
        create_if_necessary: bool,
    ) -> Option<&mut cf::ArrayOf<cf::DictionaryOfMut<cf::String, cf::Plist>>> {
        unsafe { CMSampleBufferGetSampleAttachmentsArray(self, create_if_necessary) }
    }

    #[inline]
    pub fn is_key_frame(&self) -> bool {
        match self.attaches(false) {
            Some(arr) => {
                if arr.is_empty() {
                    true
                } else {
                    let dict = &arr[0];
                    match dict.get(attach_keys::not_sync()) {
                        None => true,
                        Some(not_sync) => unsafe {
                            // in theory we don't need check actual value here.
                            // there is unsafe [`contains_not_sync()`] for faster
                            // check
                            not_sync.as_type_ptr() == cf::Boolean::value_false().as_type_ptr()
                        },
                    }
                }
            }
            None => true,
        }
    }

    #[inline]
    pub unsafe fn contains_not_sync(&self) -> bool {
        let arr = unsafe { self.attaches(true).unwrap_unchecked() };
        arr[0].contains_key(attach_keys::not_sync())
    }

    /// Returns a value that indicates whether a sample buffer is valid.
    #[doc(alias = "CMSampleBufferIsValid")]
    #[inline]
    pub fn is_valid(&self) -> bool {
        unsafe { CMSampleBufferIsValid(self) }
    }

    /// Makes the sample buffer invalid, calling any installed invalidation callback.
    ///
    /// An invalid sample buffer cannot be used -- all accessors will return kCMSampleBufferError_Invalidated.
    /// It is not a good idea to do this to a sample buffer that another module may be accessing concurrently.
    /// Example of use: the invalidation callback could cancel pending I/O.
    #[doc(alias = "CMSampleBufferInvalidate")]
    #[inline]
    pub fn invalidate(&self) -> os::Result {
        unsafe { CMSampleBufferInvalidate(self).result() }
    }

    /// Makes a cm::SampleBuf's data ready, by calling the client's
    /// cm::SampleBufferMakeDataReadyCallback.
    #[doc(alias = "CMSampleBufferMakeDataReady")]
    #[inline]
    pub fn make_data_ready(&self) -> os::Result {
        unsafe { CMSampleBufferMakeDataReady(self).result() }
    }

    /// Copies PCM audio data from the given cm::SampleBuf into
    /// a pre-populated audio::BufList. The audio::BufList must
    /// contain the same number of channels and its data buffers
    /// must be sized to hold the specified number of frames.
    /// This API is specific to audio format sample buffers, and
    /// will return kCMSampleBufferError_InvalidMediaTypeForOperation
    /// if called with a non-audio sample buffer. It will return an
    /// error if the cm::SampleBuffer does not contain PCM audio data
    /// or if its dataBuffer is not ready.
    #[cfg(feature = "cat")]
    #[doc(alias = "CMSampleBufferCopyPCMDataIntoAudioBufferList")]
    #[inline]
    pub fn copy_pcm_data_into_audio_buf_list<const N: usize>(
        &self,
        frame_offset: i32,
        num_frames: i32,
        buffer_list: &mut cat::audio::BufList<N>,
    ) -> os::Result {
        unsafe {
            CMSampleBufferCopyPCMDataIntoAudioBufferList(
                self,
                frame_offset,
                num_frames,
                std::mem::transmute(buffer_list),
            )
            .result()
        }
    }

    /// Returns a pointer to (and size of) a constant array of
    /// AudioStreamPacketDescriptions for the variable bytes per
    /// packet or variable frames per packet audio data in the
    /// provided cm::SampleBuffer.  The pointer will remain valid
    /// as long as the sbuf continues to be retained.
    /// Constant bitrate, constant frames-per-packet audio yields a
    /// return value of noErr and no packet descriptions.  This API is
    /// specific to audio format sample buffers, and will return
    /// kCMSampleBufferError_InvalidMediaTypeForOperation if called
    /// with a non-audio sample buffer.
    #[cfg(feature = "cat")]
    #[doc(alias = "CMSampleBufferGetAudioStreamPacketDescriptionsPtr")]
    #[inline]
    pub fn audio_stream_packet_descs(&self) -> os::Result<Option<&[cat::audio::StreamPacketDesc]>> {
        let ptr: *mut cat::audio::StreamPacketDesc = std::ptr::null_mut();
        let mut size = 0;
        unsafe {
            CMSampleBufferGetAudioStreamPacketDescriptionsPtr(self, ptr, &mut size).result()?;
            if ptr.is_null() {
                return Ok(None);
            }

            Ok(Some(&*slice_from_raw_parts(ptr, size)))
        }
    }

    #[cfg(feature = "cat")]
    #[doc(alias = "CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer")]
    #[inline]
    pub fn audio_buf_list<const N: usize>(&self) -> os::Result<BlockBufAudioBufList<N>> {
        self.audio_buf_list_in(
            Flags::AUDIO_BUFFER_LIST_ASSURE_16_BYTE_ALIGNMENT,
            None,
            None,
        )
    }

    #[cfg(feature = "cat")]
    #[doc(alias = "CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer")]
    #[inline]
    pub fn audio_buf_list_n<'a>(
        &self,
        list: &'a mut cat::AudioBufListN,
    ) -> os::Result<BlockBufAudioBufListN<'a>> {
        let mut buf_list_size = 0usize;
        unsafe {
            CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
                self,
                &mut buf_list_size,
                std::ptr::null_mut(),
                0,
                None,
                None,
                Flags::AUDIO_BUFFER_LIST_ASSURE_16_BYTE_ALIGNMENT,
                std::ptr::null_mut(),
            )
            .result()?;
        }

        unsafe { list.resize(buf_list_size) };
        let mut block_buf = None;

        unsafe {
            CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
                self,
                std::ptr::null_mut(),
                std::mem::transmute(list.as_mut_ptr()),
                buf_list_size,
                None,
                None,
                Flags::AUDIO_BUFFER_LIST_ASSURE_16_BYTE_ALIGNMENT,
                &mut block_buf,
            )
            .result()?;
            Ok(BlockBufAudioBufListN::new(
                list,
                block_buf.unwrap_unchecked(),
            ))
        }
    }

    /// Creates an audio::BufList containing the data from the cm::SampleBuf,
    /// and a cm::BlockBuf which references (and manages the lifetime of) the
    /// data in that audio::BufList. The data may or may not be copied,
    /// depending on the contiguity and 16-byte alignment of the cm::SampleBuffer's
    /// data. The buffers placed in the audio::BufList are guaranteed to be contiguous.
    /// The buffers in the audio::BufferList will be 16-byte-aligned if
    /// kCMSampleBufferFlag_AudioBufferList_Assure16ByteAlignment is passed in.
    #[cfg(feature = "cat")]
    #[doc(alias = "CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer")]
    #[inline]
    pub fn audio_buf_list_in<const N: usize>(
        &self,
        flags: Flags,
        block_buffer_structure_allocator: Option<&cf::Allocator>,
        block_buffer_allocator: Option<&cf::Allocator>,
    ) -> os::Result<BlockBufAudioBufList<N>> {
        let mut block_buf = None;
        let mut list = cat::audio::BufList::<N>::default();
        unsafe {
            CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
                self,
                std::ptr::null_mut(),
                std::mem::transmute(&mut list),
                std::mem::size_of::<cat::audio::BufList<N>>(),
                block_buffer_structure_allocator,
                block_buffer_allocator,
                flags,
                &mut block_buf,
            )
            .result()?;

            Ok(BlockBufAudioBufList {
                list,
                block: block_buf.unwrap_unchecked(),
            })
        }
    }

    #[doc(alias = "CMSampleBufferGetNumSamples")]
    #[inline]
    pub fn num_samples(&self) -> cf::Index {
        unsafe { CMSampleBufferGetNumSamples(self) }
    }
}
#[cfg(feature = "cat")]
#[derive(Debug)]
pub struct BlockBufAudioBufListN<'a> {
    list: &'a mut cat::AudioBufListN,
    block: arc::R<cm::BlockBuf>,
}

#[cfg(feature = "cat")]
impl<'a> BlockBufAudioBufListN<'a> {
    pub fn new(list: &'a mut cat::AudioBufListN, block: arc::R<cm::BlockBuf>) -> Self {
        Self { list, block }
    }
}

#[cfg(feature = "cat")]
#[derive(Debug)]
pub struct BlockBufAudioBufList<const N: usize> {
    list: cat::audio::BufList<N>,
    block: arc::R<cm::BlockBuf>,
}

#[cfg(feature = "cat")]
impl<const N: usize> BlockBufAudioBufList<N> {
    #[inline]
    pub fn list(&self) -> &cat::audio::BufList<N> {
        &self.list
    }

    #[inline]
    pub fn list_mut(&mut self) -> &mut cat::audio::BufList<N> {
        &mut self.list
    }

    #[inline]
    pub fn block(&self) -> &cm::BlockBuf {
        &self.block
    }
}

#[link(name = "CoreMedia", kind = "framework")]
unsafe extern "C-unwind" {
    static kCMTimingInfoInvalid: SampleTimingInfo;

    fn CMSampleBufferCreate(
        allocator: Option<&cf::Allocator>,
        data_buffer: Option<&cm::BlockBuf>,
        data_ready: bool,
        make_data_ready_cb: Option<&SampleBufMakeDataReadyCb>,
        make_data_ready_refcon: *const c_void,
        format_description: Option<&super::FormatDesc>,
        num_samples: cm::ItemCount,
        num_samples_timing_entries: cm::ItemCount,
        sample_timing_array: *const SampleTimingInfo,
        num_sample_size_entries: cm::ItemCount,
        sample_size_array: *const usize,
        sample_buffer_out: *mut Option<arc::R<SampleBuf>>,
    ) -> os::Status;

    #[cfg(feature = "cv")]
    fn CMSampleBufferCreateForImageBuffer(
        allocator: Option<&cf::Allocator>,
        image_buffer: &cv::ImageBuf,
        data_ready: bool,
        make_data_ready_cb: Option<&SampleBufMakeDataReadyCb>,
        make_data_ready_refcon: *const c_void,
        format_description: &cm::VideoFormatDesc,
        sample_timing: &SampleTimingInfo,
        sample_buffer_out: *mut Option<arc::R<SampleBuf>>,
    ) -> os::Status;

    fn CMSampleBufferDataIsReady(sbuf: &SampleBuf) -> bool;
    fn CMSampleBufferSetDataReady(sbuf: &mut SampleBuf);

    #[cfg(feature = "cv")]
    fn CMSampleBufferGetImageBuffer(sbuf: &SampleBuf) -> Option<&cv::ImageBuf>;
    fn CMSampleBufferGetDataBuffer(sbuf: &SampleBuf) -> Option<&cm::BlockBuf>;
    fn CMSampleBufferSetDataBuffer(sbuf: &mut SampleBuf, data_buffer: &cm::BlockBuf) -> os::Status;
    fn CMSampleBufferGetDuration(sbuf: &SampleBuf) -> cm::Time;
    fn CMSampleBufferGetOutputDuration(sbuf: &SampleBuf) -> cm::Time;
    fn CMSampleBufferGetPresentationTimeStamp(sbuf: &SampleBuf) -> cm::Time;
    fn CMSampleBufferGetDecodeTimeStamp(sbuf: &SampleBuf) -> cm::Time;
    fn CMSampleBufferGetOutputPresentationTimeStamp(sbuf: &SampleBuf) -> cm::Time;
    fn CMSampleBufferGetOutputDecodeTimeStamp(sbuf: &SampleBuf) -> cm::Time;
    fn CMSampleBufferSetOutputPresentationTimeStamp(sbuf: &SampleBuf, val: cm::Time);
    fn CMSampleBufferGetSampleSize(sbuf: &SampleBuf, sample_index: cm::ItemIndex) -> usize;
    fn CMSampleBufferGetTotalSampleSize(sbuf: &SampleBuf) -> usize;
    fn CMSampleBufferGetFormatDescription(sbuf: &SampleBuf) -> Option<&cm::FormatDesc>;
    fn CMSampleBufferGetSampleAttachmentsArray(
        sbuf: &SampleBuf,
        create_if_necessary: bool,
    ) -> Option<&mut cf::ArrayOf<cf::DictionaryOfMut<cf::String, cf::Plist>>>;

    fn CMSampleBufferIsValid(sbuf: &SampleBuf) -> bool;

    fn CMSampleBufferGetSampleTimingInfo(
        sbuf: &SampleBuf,
        sample_index: cm::ItemIndex,
        info: *mut cm::SampleTimingInfo,
    ) -> os::Status;

    fn CMSampleBufferInvalidate(sbuf: &SampleBuf) -> os::Status;
    fn CMSampleBufferMakeDataReady(sbuf: &SampleBuf) -> os::Status;
    #[cfg(feature = "cat")]
    fn CMSampleBufferCopyPCMDataIntoAudioBufferList(
        sbuf: &SampleBuf,
        frame_offset: i32,
        num_frames: i32,
        buffer_list: &mut cat::audio::BufList,
    ) -> os::Status;

    #[cfg(feature = "cat")]
    fn CMSampleBufferGetAudioStreamPacketDescriptionsPtr(
        sbuf: &SampleBuf,
        packet_descriptions_pointer_out: *mut cat::audio::StreamPacketDesc,
        packet_descriptions_size_out: *mut usize,
    ) -> os::Status;

    #[cfg(feature = "cat")]
    fn CMSampleBufferGetAudioBufferListWithRetainedBlockBuffer(
        sbuf: &SampleBuf,
        buffer_list_size_needed_out: *mut usize,
        buffer_list_out: *mut cat::audio::BufList,
        buffer_list_size: usize,
        block_buffer_structure_allocator: Option<&cf::Allocator>,
        block_buffer_allocator: Option<&cf::Allocator>,
        flags: Flags,
        block_buffer_out: *mut Option<arc::R<cm::BlockBuf>>,
    ) -> os::Status;

    fn CMSampleBufferGetNumSamples(sbuf: &SampleBuf) -> cf::Index;
}

/// Use attachements()
pub mod attach_keys {
    use crate::cf;

    /// cf::Boolean (absence of this key implies Sync)
    #[doc(alias = "kCMSampleAttachmentKey_NotSync")]
    #[inline]
    pub fn not_sync() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_NotSync }
    }

    /// cf::Boolean (absence of this key implies not Partial Sync. If NotSync is false, PartialSync should be ignored.)
    #[doc(alias = "kCMSampleAttachmentKey_PartialSync")]
    #[inline]
    pub fn partial_sync() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_PartialSync }
    }

    /// kCFBooleanTrue, kCFBooleanFalse, or absent if unknown
    #[doc(alias = "kCMSampleAttachmentKey_HasRedundantCoding")]
    pub fn has_redundant_coding() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_HasRedundantCoding }
    }

    /// kCFBooleanTrue, kCFBooleanFalse, or absent if unknown
    ///
    /// A frame is considered droppable if and only if kCMSampleAttachmentKey_IsDependedOnByOthers is present and set to kCFBooleanFalse.
    #[doc(alias = "kCMSampleAttachmentKey_IsDependedOnByOthers")]
    #[inline]
    pub fn is_depended_on_by_others() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_IsDependedOnByOthers }
    }

    /// cf::Boolean::value_true() (e.g., non-I-frame), cf::Boolean::value_false() (e.g. I-frame), or absent if unknown
    #[doc(alias = "kCMSampleAttachmentKey_DependsOnOthers")]
    #[inline]
    pub fn depends_on_others() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_DependsOnOthers }
    }

    /// cf::Boolean
    #[doc(alias = "kCMSampleAttachmentKey_EarlierDisplayTimesAllowed")]
    #[inline]
    pub fn earlier_display_times_allowed() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_EarlierDisplayTimesAllowed }
    }

    /// cf::Boolean
    #[doc(alias = "kCMSampleAttachmentKey_DisplayImmediately")]
    #[inline]
    pub fn display_immediately() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_DisplayImmediately }
    }

    /// cf::Boolean
    #[doc(alias = "kCMSampleAttachmentKey_DoNotDisplay")]
    #[inline]
    pub fn do_not_display() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_DoNotDisplay }
    }

    #[doc(alias = "kCMSampleAttachmentKey_CryptorSubsampleAuxiliaryData")]
    #[inline]
    pub fn cryptor_subsample_auxiliary_data() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_CryptorSubsampleAuxiliaryData }
    }

    ///  HDR10+ per frame metadata
    ///
    /// The attachment is cf::Data containing HDR10+ metadata within an User Data Registered
    /// ITU-T T-35 SEI message (see ISO/IEC 23008-2-2020 section D.3.6) as little endian in the cf::Data.
    /// This attachment will override any HDR10+ metadata stored within the compressed data.
    /// The data shall start with the field itu_t_t35_country_code with the value 0xb5.
    #[doc(alias = "kCMSampleAttachmentKey_HDR10PlusPerFrameData")]
    #[inline]
    pub fn hdr10plus_per_frame_data() -> &'static cf::String {
        unsafe { kCMSampleAttachmentKey_HDR10PlusPerFrameData }
    }

    // https://developer.apple.com/library/archive/qa/qa1957/_index.html#//apple_ref/doc/uid/DTS40017660
    unsafe extern "C" {
        static kCMSampleAttachmentKey_NotSync: &'static cf::String;
        static kCMSampleAttachmentKey_PartialSync: &'static cf::String;
        static kCMSampleAttachmentKey_HasRedundantCoding: &'static cf::String;
        static kCMSampleAttachmentKey_IsDependedOnByOthers: &'static cf::String;
        static kCMSampleAttachmentKey_DependsOnOthers: &'static cf::String;
        static kCMSampleAttachmentKey_EarlierDisplayTimesAllowed: &'static cf::String;
        static kCMSampleAttachmentKey_DisplayImmediately: &'static cf::String;
        static kCMSampleAttachmentKey_DoNotDisplay: &'static cf::String;
        static kCMSampleAttachmentKey_CryptorSubsampleAuxiliaryData: &'static cf::String;
        static kCMSampleAttachmentKey_HDR10PlusPerFrameData: &'static cf::String;
    }
}

/// use get_attachment or dictionary_of_attachments
pub mod buf_attach_keys {
    use crate::cf;

    /// cf::Boolean
    #[inline]
    pub fn reset_decoder_before_decoding() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_ResetDecoderBeforeDecoding }
    }

    /// cf::Boolean
    #[inline]
    pub fn drain_after_decoding() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_DrainAfterDecoding }
    }

    /// cf::Dictionary (client-defined)
    #[inline]
    pub fn post_notification_when_consumed() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_PostNotificationWhenConsumed }
    }

    /// 'cf::Number' (ResumeTag)
    #[inline]
    pub fn resume_output() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_ResumeOutput }
    }

    /// Marks a transition from one source of buffers (eg. song) to another
    ///
    /// For example, during gapless playback of a list of songs, this attachment marks the first buffer from the next song.
    /// If this attachment is on a buffer containing no samples, the first following buffer that contains samples is the
    /// buffer that contains the first samples from the next song.  The value of this attachment is a CFTypeRef.  This
    /// transition identifier should be unique within a playlist, so each transition in a playlist is uniquely
    /// identifiable. A cf::Number counter that increments with each transition is a simple example.
    #[inline]
    pub fn transition_id() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_TransitionID }
    }

    /// The duration that should be removed at the beginning of the sample buffer, after decoding.
    #[inline]
    pub fn trim_duration_at_start() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_TrimDurationAtStart }
    }

    /// The duration that should be removed at the end of the sample buffer, after decoding.
    ///
    /// If this attachment is not present, the trim duration is zero (nothing removed).
    /// This is a cm::Time in cf::Dictionary format as made by CMTimeCopyAsDictionary;
    /// use CMTimeMakeFromDictionary to convert to cm::Time.
    #[inline]
    pub fn trim_duration_at_end() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_TrimDurationAtEnd }
    }

    /// Indicates that the decoded contents of the sample buffer should be reversed.
    ///
    /// If this attachment is not present, the sample buffer should be played forwards as usual.
    /// Reversal occurs after trimming and speed multipliers.
    #[inline]
    pub fn reverse() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_Reverse }
    }

    /// Fill the difference between discontiguous sample buffers with silence
    ///
    /// If a sample buffer enters a buffer queue and the presentation time stamp between the
    /// previous buffer and the buffer with this attachment are discontiguous, handle the
    /// discontinuity by generating silence for the time difference.
    #[inline]
    pub fn fill_discontinuities_with_silence() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_FillDiscontinuitiesWithSilence }
    }

    /// Marks an intentionally empty interval in the sequence of samples.
    ///
    /// The sample buffer's output presentation timestamp indicates when the empty interval begins.
    /// Marker sample buffers with this attachment are used to announce the arrival of empty edits.
    #[inline]
    pub fn empty_media() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_EmptyMedia }
    }

    /// Marks the end of the sequence of samples.
    ///
    /// Marker sample buffers with this attachment in addition to kCMSampleBufferAttachmentKey_EmptyMedia
    /// are used to indicate that no further samples are expected.
    #[inline]
    pub fn permanent_empty_media() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_PermanentEmptyMedia }
    }

    /// Tells that the empty marker should be dequeued immediately regardless of its timestamp.
    ///
    /// Marker sample buffers with this attachment in addition to kCMSampleBufferAttachmentKey_EmptyMedia
    /// are used to tell that the empty sample buffer should be dequeued immediately regardless of its timestamp.
    /// This attachment should only be used with sample buffers with the kCMSampleBufferAttachmentKey_EmptyMedia
    /// attachment.
    #[inline]
    pub fn display_empty_media_immediately() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_DisplayEmptyMediaImmediately }
    }

    /// Indicates that sample buffer's decode timestamp may be used to define the previous sample buffer's duration.
    ///
    /// Marker sample buffers with this attachment may be used in situations where sample buffers are transmitted
    /// before their duration is known. In such situations, normally the recipient may use each sample buffer's timestamp
    /// to calculate the duration of the previous sample buffer. The marker sample buffer with this attachment is sent
    /// to provide the timestamp for calculating the final sample buffer's duration.
    #[inline]
    pub fn ends_previous_sample_duration() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_EndsPreviousSampleDuration }
    }

    /// Indicates the URL where the sample data is.
    #[inline]
    pub fn sample_reference_url() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_SampleReferenceURL }
    }

    /// Indicates the byte offset at which the sample data begins.
    #[inline]
    pub fn sample_reference_byte_offset() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_SampleReferenceByteOffset }
    }

    /// Indicates the reason the current video frame was dropped.
    ///
    /// Sample buffers with this attachment contain no image or data buffer.  They mark a dropped video
    /// frame.  This attachment identifies the reason for the droppage.
    #[inline]
    pub fn dropped_frame_reason() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_DroppedFrameReason }
    }
    /// Indicates additional information regarding the dropped video frame.
    #[inline]
    pub fn dropped_frame_reason_info() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_DroppedFrameReasonInfo }
    }
    /// Indicates information about the lens stabilization applied to the current still image buffer.
    ///
    /// Sample buffers that have been captured with a lens stabilization module may have an attachment of
    /// kCMSampleBufferAttachmentKey_StillImageLensStabilizationInfo which has information about the stabilization status
    /// during the capture.  This key will not be present in CMSampleBuffers coming from cameras without a lens stabilization module.
    #[inline]
    pub fn still_image_lens_stabilization_info() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_StillImageLensStabilizationInfo }
    }

    /// Indicates the 3x3 camera intrinsic matrix applied to the current sample buffer.
    ///
    /// Camera intrinsic matrix is a CFData containing a matrix_float3x3, which is column-major. It has the following contents:
    /// fx   0    ox
    /// 0    fy   oy
    /// 0    0    1
    /// fx and fy are the focal length in pixels. For square pixels, they will have the same value.
    /// ox and oy are the coordinates of the principal point. The origin is the upper left of the frame.
    #[inline]
    pub fn camera_intrinsic_matrix() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_CameraIntrinsicMatrix }
    }

    /// Indicates that the current or next video sample buffer should be forced to be encoded as a key frame.
    ///
    /// A value of kCFBooleanTrue for kCMSampleBufferAttachmentKey_ForceKeyFrame indicates
    /// that the current or next video sample buffer processed in the stream should be forced
    /// to be encoded as a key frame.
    /// If this attachment is present and kCFBooleanTrue on a sample buffer with a video
    /// frame, that video frame will be forced to become a key frame.  If the sample
    /// buffer for which this is present and kCFBooleanTrue does not have a valid video
    /// frame, the next sample buffer processed that contains a valid video frame will be
    /// encoded as a key frame.
    #[inline]
    pub fn force_key_frame() -> &'static cf::String {
        unsafe { kCMSampleBufferAttachmentKey_ForceKeyFrame }
    }

    // https://developer.apple.com/library/archive/qa/qa1957/_index.html#//apple_ref/doc/uid/DTS40017660
    unsafe extern "C" {
        static kCMSampleBufferAttachmentKey_ResetDecoderBeforeDecoding: &'static cf::String;
        static kCMSampleBufferAttachmentKey_DrainAfterDecoding: &'static cf::String;
        static kCMSampleBufferAttachmentKey_PostNotificationWhenConsumed: &'static cf::String;
        static kCMSampleBufferAttachmentKey_ResumeOutput: &'static cf::String;

        static kCMSampleBufferAttachmentKey_TransitionID: &'static cf::String;
        static kCMSampleBufferAttachmentKey_TrimDurationAtStart: &'static cf::String;
        static kCMSampleBufferAttachmentKey_TrimDurationAtEnd: &'static cf::String;
        static kCMSampleBufferAttachmentKey_Reverse: &'static cf::String;
        static kCMSampleBufferAttachmentKey_FillDiscontinuitiesWithSilence: &'static cf::String;
        static kCMSampleBufferAttachmentKey_EmptyMedia: &'static cf::String;
        static kCMSampleBufferAttachmentKey_PermanentEmptyMedia: &'static cf::String;
        static kCMSampleBufferAttachmentKey_DisplayEmptyMediaImmediately: &'static cf::String;
        static kCMSampleBufferAttachmentKey_EndsPreviousSampleDuration: &'static cf::String;
        static kCMSampleBufferAttachmentKey_SampleReferenceURL: &'static cf::String;
        static kCMSampleBufferAttachmentKey_SampleReferenceByteOffset: &'static cf::String;
        static kCMSampleBufferAttachmentKey_DroppedFrameReason: &'static cf::String;
        static kCMSampleBufferAttachmentKey_DroppedFrameReasonInfo: &'static cf::String;
        static kCMSampleBufferAttachmentKey_StillImageLensStabilizationInfo: &'static cf::String;
        static kCMSampleBufferAttachmentKey_CameraIntrinsicMatrix: &'static cf::String;
        static kCMSampleBufferAttachmentKey_ForceKeyFrame: &'static cf::String;
    }
}

pub mod err {
    use crate::os::Error;

    /// An allocation failed.
    #[doc(alias = "kCMSampleBufferError_AllocationFailed")]
    pub const ALLOC_FAILED: Error = Error::new_unchecked(-12730);

    /// NULL or 0 was passed for a required parameter.
    #[doc(alias = "kCMSampleBufferError_RequiredParameterMissing")]
    pub const REQUIRED_PARAMETER_MISSING: Error = Error::new_unchecked(-12731);

    /// Attempt was made to set a dataBuffer on a cm::SampleBuffer that already has one.
    #[doc(alias = "kCMSampleBufferError_AlreadyHasDataBuffer")]
    pub const ALREADY_HAS_DATA_BUFFER: Error = Error::new_unchecked(-12732);

    /// Buffer could not be made ready.
    #[doc(alias = "kCMSampleBufferError_BufferNotReady")]
    pub const BUFFER_NOT_READY: Error = Error::new_unchecked(-12733);

    /// Sample index was not between 0 and numSamples-1, inclusive.
    #[doc(alias = "kCMSampleBufferError_SampleIndexOutOfRange")]
    pub const SAMPLE_INDEX_OUT_OF_RANGE: Error = Error::new_unchecked(-12734);

    /// Attempt to get sample size information when there was none.
    #[doc(alias = "kCMSampleBufferError_BufferHasNoSampleSizes")]
    pub const BUFFER_HAS_NO_SAMPLE_SIZES: Error = Error::new_unchecked(-12735);

    /// Attempt to get sample timing information when there was none.
    #[doc(alias = "kCMSampleBufferError_BufferHasNoSampleTimingInfo")]
    pub const BUFFER_HAS_NO_SAMPLE_TIMING_INFO: Error = Error::new_unchecked(-12736);

    /// Output array was not large enough for the array being requested.
    #[doc(alias = "kCMSampleBufferError_ArrayTooSmall")]
    pub const ARRAY_TOO_SMALL: Error = Error::new_unchecked(-12737);

    /// Timing info or size array entry count was not 0, 1, or numSamples.
    #[doc(alias = "kCMSampleBufferError_InvalidEntryCount")]
    pub const INVALID_ENTRY_COUNT: Error = Error::new_unchecked(-12738);

    /// Sample buffer does not contain sample sizes.  This can happen when the samples in the buffer are non-contiguous (eg. non-interleaved audio, where the channel values for a single sample are scattered through the buffer).
    #[doc(alias = "kCMSampleBufferError_CannotSubdivide")]
    pub const CANNOT_SUBDIVIDE: Error = Error::new_unchecked(-12739);

    /// buffer unexpectedly contains a non-numeric sample timing info
    #[doc(alias = "kCMSampleBufferError_SampleTimingInfoInvalid")]
    pub const SAMPLE_TIMING_INFO_INVALID: Error = Error::new_unchecked(-12740);

    /// the media type specified by a format description is not valid for the given
    /// operation (eg. a cm::SampleBuffer with a non-audio format description passed
    /// to cm::SampleBufferGetAudioStreamPacketDescriptionsPtr).
    #[doc(alias = "kCMSampleBufferError_InvalidMediaTypeForOperation")]
    pub const INVALID_MEDIA_TYPE_FOR_OPERATION: Error = Error::new_unchecked(-12741);

    /// Buffer contains bad data. Only returned by cm::SampleBuffer functions
    /// that inspect its sample data.
    #[doc(alias = "kCMSampleBufferError_InvalidSampleData")]
    pub const INVALID_SAMPLE_DATA: Error = Error::new_unchecked(-12742);

    /// The format of the given media does not match the given format description
    /// (eg. a format description paired with a cv::ImageBuffer that fails
    /// cm::VideoFormatDescriptionMatchesImageBuffer).
    #[doc(alias = "kCMSampleBufferError_InvalidMediaFormat")]
    pub const INVALID_MEDIA_FORMAT: Error = Error::new_unchecked(-12743);

    /// the sample buffer was invalidated.
    #[doc(alias = "kCMSampleBufferError_Invalidated")]
    pub const INVALIDATED: Error = Error::new_unchecked(-12744);

    /// the sample buffer's data loading operation failed (generic error).
    #[doc(alias = "kCMSampleBufferError_DataFailed")]
    pub const DATA_FAILED: Error = Error::new_unchecked(-16750);

    /// the sample buffer's data loading operation was canceled.
    #[doc(alias = "kCMSampleBufferError_DataCanceled")]
    pub const DATA_CANCELED: Error = Error::new_unchecked(-16751);
}

#[cfg(test)]
mod tests {
    use crate::{cf, cm, cv};

    #[test]
    fn basics() {
        let image_buf = cv::ImageBuf::new(1920, 1080, cv::PixelFormat::_32_BGRA, None).unwrap();
        let format_desc = cm::FormatDesc::with_image_buf(&image_buf).unwrap();
        let timing_info = cm::SampleTimingInfo::invalid();
        let mut sample_buf = cm::SampleBuf::with_image_buf(
            &image_buf,
            true,
            None,
            std::ptr::null(),
            &format_desc,
            &timing_info,
        )
        .unwrap();
        assert!(sample_buf.data_is_ready());

        let attaches = sample_buf.attaches_mut(true).unwrap();
        attaches[0].insert(
            cm::sample_buffer::attach_keys::do_not_display(),
            cf::Number::tagged_i8(1).into(),
        );

        assert!(sample_buf.is_key_frame());

        let err = sample_buf
            .audio_stream_packet_descs()
            .expect_err("It is video format");
        assert_eq!(err, cm::sample_buf_err::INVALID_MEDIA_TYPE_FOR_OPERATION);

        // check with match:
        match sample_buf.audio_stream_packet_descs() {
            Err(cm::sample_buf_err::INVALID_MEDIA_TYPE_FOR_OPERATION) => {}
            _ => {
                panic!("should be error")
            }
        }
    }
}
