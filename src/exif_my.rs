use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{Value, Map, json};
use std::io::Cursor;
use image::codecs::jpeg::JpegEncoder;

#[macro_export]
macro_rules! apply_exif_tags {
    ($callback:ident) => {
        $callback!((), [
            (InteropIndex,0x0001,"InteropIndex"), 
            (InteropVersion,0x0002,"InteropVersion"),
            (ImageWidth,0x0100,"ImageWidth"),
            (ImageLength,0x0101,"ImageLength"),
            (BitsPerSample,0x0102,"BitsPerSample"),
            (Compression,0x0103,"Compression"),
            (PhotometricInterpretation,0x0106,"PhotometricInterpretation"),
            (FillOrder,0x010A,"FillOrder"),
            (DocumentName,0x010D,"DocumentName"),
            (ImageDescription,0x010E,"ImageDescription"),
            (Make,0x010F,"Make"),
            (Model,0x0110,"Model"),
            (StripOffsets,0x0111,"StripOffsets"),
            (Orientation,0x0112,"Orientation"),
            (SamplesPerPixel,0x0115,"SamplesPerPixel"),
            (RowsPerStrip,0x0116,"RowsPerStrip"),
            (StripByteCounts,0x0117,"StripByteCounts"),
            (XResolution,0x011A,"XResolution"),
            (YResolution,0x011B,"YResolution"),
            (PlanarConfiguration,0x011C,"PlanarConfiguration"),
            (ResolutionUnit,0x0128,"ResolutionUnit"),
            (TransferFunction,0x012D,"TransferFunction"),//3*256*u8 LUT
            (Software,0x0131,"Software"),
            (DateTime,0x0132,"DateTime"),
            (Artist,0x013B,"Artist"),
            (WhitePoint,0x013E,"WhitePoint"),
            (PrimaryChromaticities,0x013F,"PrimaryChromaticities"),
            (TransferRange,0x0156,"TransferRange"),
            (JPEGProc,0x0200,"JPEGProc"),
            (ThumbnailOffset,0x0201,"ThumbnailOffset"),//JPEGInterchangeFormat
            (ThumbnailLength,0x0202,"ThumbnailLength"),//JPEGInterchangeFormatLength
            (YCbCrCoefficients,0x0211,"YCbCrCoefficients"),
            (YCbCrSubSampling,0x0212,"YCbCrSubSampling"),
            (YCbCrPositioning,0x0213,"YCbCrPositioning"),
            (ReferenceBlackWhite,0x0214,"ReferenceBlackWhite"),
            (RelatedImageWidth,0x1001,"RelatedImageWidth"),
            (RelatedImageLength,0x1002,"RelatedImageLength"),
            (CFARepeatPatternDim,0x828D,"CFARepeatPatternDim"),
            (BatteryLevel,0x828F,"BatteryLevel"),
            (Copyright,0x8298,"Copyright"),
            (ExposureTime,0x829A,"ExposureTime"),
            (FNUMBER,0x829D,"FNumber"),
            (IPTC_NAA,0x83BB,"IPTC/NAA"),
            (EXIF_OFFSET,0x8769,"ExifOffset"),//Exif IFD Pointer
            (InterColorProfile,0x8773,"InterColorProfile"),
            (EXPOSURE_PROGRAM,0x8822,"ExposureProgram"),
            (SpectralSensitivity,0x8824,"SpectralSensitivity"),
            (GPSInfo,0x8825,"GPSInfo"),//GPS Info IFD Pointer
            (ISO_EQUIVALENT,0x8827,"ISOSpeedRatings"),
            (OECF,0x8828,"OECF"),
            (SensitivityType,0x8830,"SensitivityType"),
            (StandardOutputSensitivity,0x8831,"StandardOutputSensitivity"),
            (RecommendedExposureIndex,0x8832,"RecommendedExposureIndex"),
            (ISOSpeed,0x8833,"ISOSpeed"),
            (ISOSpeedLatitudeyyy,0x8834,"ISOSpeedLatitudeyyy"),
            (ISOSpeedLatitudezzz,0x8835,"ISOSpeedLatitudezzz"),
            (ExifVersion,0x9000,"ExifVersion"),
            (DateTimeOriginal,0x9003,"DateTimeOriginal"),
            (DateTimeDigitized,0x9004,"DateTimeDigitized"),
            (OffsetTime,0x9010,"OffsetTime"),
            (OffsetTimeOriginal,0x9011,"OffsetTimeOriginal"),
            (OffsetTimeDigitized ,0x9012,"OffsetTimeDigitized "),
            (ComponentsConfiguration,0x9101,"ComponentsConfiguration"),
            (CompressedBitsPerPixel,0x9102,"CompressedBitsPerPixel"),
            (ShutterSpeedValue,0x9201,"ShutterSpeedValue"),
            (ApertureValue,0x9202,"ApertureValue"),
            (BrightnessValue,0x9203,"BrightnessValue"),
            (ExposureBiasValue,0x9204,"ExposureBiasValue"),
            (MaxApertureValue,0x9205,"MaxApertureValue"),
            (SubjectDistance,0x9206,"SubjectDistance"),
            (MeteringMode,0x9207,"MeteringMode"),
            (LightSource,0x9208,"LightSource"),
            (Flash,0x9209,"Flash"),
            (FocalLength,0x920A,"FocalLength"),
            (FlashEnergy_,0x920B,"FlashEnergy"),
            (SpatialFrequencyResponse_,0x920C,"SpatialFrequencyResponse"),
            (FOCALPLANEXRES_,0x920E,"FocalPlaneXResolution"),
            (FocalPlaneYResolution_,0x920F,"FocalPlaneYResolution"),
            (FOCALPLANEUNITS_,0x9210,"FocalPlaneResolutionUnit"),
            (ExposureIndex,0x9215,"ExposureIndex"),
            (SensingMethod_,0x9217,"SensingMethod"),
            (MakerNote,0x927C,"MakerNote"),
            (UserComment,0x9286,"UserComment"),
            (SubSecTime,0x9290,"SubSecTime"),
            (SubSecTimeOriginal,0x9291,"SubSecTimeOriginal"),
            (SubSecTimeDigitized,0x9292,"SubSecTimeDigitized"),
            (Temperature,0x9400,"Temperature"),
            (Humidity,0x9401,"Humidity"),
            (Pressure,0x9402,"Pressure"),
            (WaterDepth,0x9403,"WaterDepth"),
            (Acceleration,0x9404,"Acceleration"),
            (CameraElevationAngle,0x9405,"CameraElevationAngle"),
            (FlashPixVersion,0xA000,"FlashPixVersion"),
            (ColorSpace,0xA001,"ColorSpace"),
            (PixelXDimension,0xa002,"PixelXDimension"),
            (PixelYDimension,0xa003,"PixelYDimension"),
            (RelatedAudioFile,0xA004,"RelatedAudioFile"),
            (INTEROP_OFFSET,0xa005,"InteroperabilityOffset"),//Interoperability IFD Pointer
            (FlashEnergy,0xA20B,"FlashEnergy"),
            (SpatialFrequencyResponse,0xA20C,"SpatialFrequencyResponse"),
            (FOCALPLANEXRES,0xa20E,"FocalPlaneXResolution"),
            (FocalPlaneYResolution,0xA20F,"FocalPlaneYResolution"),
            (FOCALPLANEUNITS,0xa210,"FocalPlaneResolutionUnit"),
            (SubjectLocation,0xA214,"SubjectLocation"),
            (EXPOSURE_INDEX,0xa215,"ExposureIndex"),
            (SensingMethod,0xA217,"SensingMethod"),
            (FileSource,0xA300,"FileSource"),
            (SceneType,0xA301,"SceneType"),
            (CFAPattern,0xA302,"CFAPattern"),
            (CustomRendered,0xa401,"CustomRendered"),
            (ExposureMode,0xa402,"ExposureMode"),
            (WhiteBalance,0xa403,"WhiteBalance"),
            (DigitalZoomRatio,0xa404,"DigitalZoomRatio"),
            (FOCALLENGTH_35MM,0xa405,"FocalLengthIn35mmFilm"),
            (SceneCaptureType,0xa406,"SceneCaptureType"),
            (GainControl,0xa407,"GainControl"),
            (Contrast,0xa408,"Contrast"),
            (Saturation, 0xa409, "Saturation"),
            (Sharpness, 0xa40a, "Sharpness"),
            (DeviceSettingDescription, 0xa40b, "DeviceSettingDescription"),
            (SubjectDistanceRange, 0xa40c, "SubjectDistanceRange"),
            (ImageUniqueID, 0xa420, "ImageUniqueID"),
            (CameraOwnerName, 0xa430, "CameraOwnerName"),
            (BodySerialNumber, 0xa431, "BodySerialNumber"),
            (LensSpecification, 0xa432, "LensSpecification"),
            (LensMake, 0xa433, "LensMake"),
            (LensModel, 0xa434, "LensModel"),
            (LensSerialNumber, 0xa435, "LensSerialNumber"),
            (ImageTitle, 0xa436, "ImageTitle"),
            (Photographer, 0xa437, "Photographer"),
            (ImageEditor, 0xa438, "ImageEditor"),
            (CameraFirmware, 0xa439, "CameraFirmware"),
            (RAWDevelopingSoftware, 0xa43a, "RAWDevelopingSoftware"),
            (ImageEditingSoftware, 0xa43b, "ImageEditingSoftware"),
            (MetadataEditingSoftware, 0xa43c, "MetadataEditingSoftware"),
            (CompositeImage, 0xa460, "CompositeImage"),
            (SourceImageNumberOfCompositeImage, 0xa461, "SourceImageNumberOfCompositeImage"),
            (SourceExposureTimesOfCompositeImage, 0xa462, "SourceExposureTimesOfCompositeImage"),
            (Gamma, 0xa500, "Gamma"),
            (UndefinedExifTag,0xffff,"UndefinedExifTag")
        ]);
    };

    ($callback:ident, $extra:expr) => {
        $callback!($extra, [
            (InteropIndex,0x0001,"InteropIndex"), 
            (InteropVersion,0x0002,"InteropVersion"),
            (ImageWidth,0x0100,"ImageWidth"),
            (ImageLength,0x0101,"ImageLength"),
            (BitsPerSample,0x0102,"BitsPerSample"),
            (Compression,0x0103,"Compression"),
            (PhotometricInterpretation,0x0106,"PhotometricInterpretation"),
            (FillOrder,0x010A,"FillOrder"),
            (DocumentName,0x010D,"DocumentName"),
            (ImageDescription,0x010E,"ImageDescription"),
            (Make,0x010F,"Make"),
            (Model,0x0110,"Model"),
            (StripOffsets,0x0111,"StripOffsets"),
            (Orientation,0x0112,"Orientation"),
            (SamplesPerPixel,0x0115,"SamplesPerPixel"),
            (RowsPerStrip,0x0116,"RowsPerStrip"),
            (StripByteCounts,0x0117,"StripByteCounts"),
            (XResolution,0x011A,"XResolution"),
            (YResolution,0x011B,"YResolution"),
            (PlanarConfiguration,0x011C,"PlanarConfiguration"),
            (ResolutionUnit,0x0128,"ResolutionUnit"),
            (TransferFunction,0x012D,"TransferFunction"),//3*256*u8 LUT
            (Software,0x0131,"Software"),
            (DateTime,0x0132,"DateTime"),
            (Artist,0x013B,"Artist"),
            (WhitePoint,0x013E,"WhitePoint"),
            (PrimaryChromaticities,0x013F,"PrimaryChromaticities"),
            (TransferRange,0x0156,"TransferRange"),
            (JPEGProc,0x0200,"JPEGProc"),
            (ThumbnailOffset,0x0201,"ThumbnailOffset"),//JPEGInterchangeFormat
            (ThumbnailLength,0x0202,"ThumbnailLength"),//JPEGInterchangeFormatLength
            (YCbCrCoefficients,0x0211,"YCbCrCoefficients"),
            (YCbCrSubSampling,0x0212,"YCbCrSubSampling"),
            (YCbCrPositioning,0x0213,"YCbCrPositioning"),
            (ReferenceBlackWhite,0x0214,"ReferenceBlackWhite"),
            (RelatedImageWidth,0x1001,"RelatedImageWidth"),
            (RelatedImageLength,0x1002,"RelatedImageLength"),
            (CFARepeatPatternDim,0x828D,"CFARepeatPatternDim"),
            (BatteryLevel,0x828F,"BatteryLevel"),
            (Copyright,0x8298,"Copyright"),
            (ExposureTime,0x829A,"ExposureTime"),
            (FNUMBER,0x829D,"FNumber"),
            (IPTC_NAA,0x83BB,"IPTC/NAA"),
            (EXIF_OFFSET,0x8769,"ExifOffset"),//Exif IFD Pointer
            (InterColorProfile,0x8773,"InterColorProfile"),
            (EXPOSURE_PROGRAM,0x8822,"ExposureProgram"),
            (SpectralSensitivity,0x8824,"SpectralSensitivity"),
            (GPSInfo,0x8825,"GPSInfo"),//GPS Info IFD Pointer
            (ISO_EQUIVALENT,0x8827,"ISOSpeedRatings"),
            (OECF,0x8828,"OECF"),
            (SensitivityType,0x8830,"SensitivityType"),
            (StandardOutputSensitivity,0x8831,"StandardOutputSensitivity"),
            (RecommendedExposureIndex,0x8832,"RecommendedExposureIndex"),
            (ISOSpeed,0x8833,"ISOSpeed"),
            (ISOSpeedLatitudeyyy,0x8834,"ISOSpeedLatitudeyyy"),
            (ISOSpeedLatitudezzz,0x8835,"ISOSpeedLatitudezzz"),
            (ExifVersion,0x9000,"ExifVersion"),
            (DateTimeOriginal,0x9003,"DateTimeOriginal"),
            (DateTimeDigitized,0x9004,"DateTimeDigitized"),
            (OffsetTime,0x9010,"OffsetTime"),
            (OffsetTimeOriginal,0x9011,"OffsetTimeOriginal"),
            (OffsetTimeDigitized ,0x9012,"OffsetTimeDigitized "),
            (ComponentsConfiguration,0x9101,"ComponentsConfiguration"),
            (CompressedBitsPerPixel,0x9102,"CompressedBitsPerPixel"),
            (ShutterSpeedValue,0x9201,"ShutterSpeedValue"),
            (ApertureValue,0x9202,"ApertureValue"),
            (BrightnessValue,0x9203,"BrightnessValue"),
            (ExposureBiasValue,0x9204,"ExposureBiasValue"),
            (MaxApertureValue,0x9205,"MaxApertureValue"),
            (SubjectDistance,0x9206,"SubjectDistance"),
            (MeteringMode,0x9207,"MeteringMode"),
            (LightSource,0x9208,"LightSource"),
            (Flash,0x9209,"Flash"),
            (FocalLength,0x920A,"FocalLength"),
            (FlashEnergy_,0x920B,"FlashEnergy"),
            (SpatialFrequencyResponse_,0x920C,"SpatialFrequencyResponse"),
            (FOCALPLANEXRES_,0x920E,"FocalPlaneXResolution"),
            (FocalPlaneYResolution_,0x920F,"FocalPlaneYResolution"),
            (FOCALPLANEUNITS_,0x9210,"FocalPlaneResolutionUnit"),
            (ExposureIndex,0x9215,"ExposureIndex"),
            (SensingMethod_,0x9217,"SensingMethod"),
            (MAKER_NOTE,0x927C,"MakerNote"),
            (USERCOMMENT,0x9286,"UserComment"),
            (SubSecTime,0x9290,"SubSecTime"),
            (SubSecTimeOriginal,0x9291,"SubSecTimeOriginal"),
            (SubSecTimeDigitized,0x9292,"SubSecTimeDigitized"),
            (Temperature,0x9400,"Temperature"),
            (Humidity,0x9401,"Humidity"),
            (Pressure,0x9402,"Pressure"),
            (WaterDepth,0x9403,"WaterDepth"),
            (Acceleration,0x9404,"Acceleration"),
            (CameraElevationAngle,0x9405,"CameraElevationAngle"),
            (FlashPixVersion,0xA000,"FlashPixVersion"),
            (ColorSpace,0xA001,"ColorSpace"),
            (PixelXDimension,0xa002,"PixelXDimension"),
            (PixelYDimension,0xa003,"PixelYDimension"),
            (RelatedAudioFile,0xA004,"RelatedAudioFile"),
            (INTEROP_OFFSET,0xa005,"InteroperabilityOffset"),//Interoperability IFD Pointer
            (FlashEnergy,0xA20B,"FlashEnergy"),
            (SpatialFrequencyResponse,0xA20C,"SpatialFrequencyResponse"),
            (FOCALPLANEXRES,0xa20E,"FocalPlaneXResolution"),
            (FocalPlaneYResolution,0xA20F,"FocalPlaneYResolution"),
            (FOCALPLANEUNITS,0xa210,"FocalPlaneResolutionUnit"),
            (SubjectLocation,0xA214,"SubjectLocation"),
            (EXPOSURE_INDEX,0xa215,"ExposureIndex"),
            (SensingMethod,0xA217,"SensingMethod"),
            (FileSource,0xA300,"FileSource"),
            (SceneType,0xA301,"SceneType"),
            (CFAPattern,0xA302,"CFAPattern"),
            (CustomRendered,0xa401,"CustomRendered"),
            (ExposureMode,0xa402,"ExposureMode"),
            (WhiteBalance,0xa403,"WhiteBalance"),
            (DigitalZoomRatio,0xa404,"DigitalZoomRatio"),
            (FOCALLENGTH_35MM,0xa405,"FocalLengthIn35mmFilm"),
            (SceneCaptureType,0xa406,"SceneCaptureType"),
            (GainControl,0xa407,"GainControl"),
            (Contrast,0xa408,"Contrast"),
            (Saturation, 0xa409, "Saturation"),
            (Sharpness, 0xa40a, "Sharpness"),
            (DeviceSettingDescription, 0xa40b, "DeviceSettingDescription"),
            (SubjectDistanceRange, 0xa40c, "SubjectDistanceRange"),
            (ImageUniqueID, 0xa420, "ImageUniqueID"),
            (CameraOwnerName, 0xa430, "CameraOwnerName"),
            (BodySerialNumber, 0xa431, "BodySerialNumber"),
            (LensSpecification, 0xa432, "LensSpecification"),
            (LensMake, 0xa433, "LensMake"),
            (LensModel, 0xa434, "LensModel"),
            (LensSerialNumber, 0xa435, "LensSerialNumber"),
            (ImageTitle, 0xa436, "ImageTitle"),
            (Photographer, 0xa437, "Photographer"),
            (ImageEditor, 0xa438, "ImageEditor"),
            (CameraFirmware, 0xa439, "CameraFirmware"),
            (RAWDevelopingSoftware, 0xa43a, "RAWDevelopingSoftware"),
            (ImageEditingSoftware, 0xa43b, "ImageEditingSoftware"),
            (MetadataEditingSoftware, 0xa43c, "MetadataEditingSoftware"),
            (CompositeImage, 0xa460, "CompositeImage"),
            (SourceImageNumberOfCompositeImage, 0xa461, "SourceImageNumberOfCompositeImage"),
            (SourceExposureTimesOfCompositeImage, 0xa462, "SourceExposureTimesOfCompositeImage"),
            (Gamma, 0xa500, "Gamma"),
            (UndefinedExifTag,0xffff,"UndefinedExifTag")
        ])
    };
}

macro_rules! make_exif_enum {
    ($unused:tt, [ $( ($name:ident, $id:expr, $str:expr) ),* ]) => {
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        #[repr(u16)]
        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        pub enum ExifTagId {
            $( $name = $id ),*
        }
    };
}
apply_exif_tags!{ make_exif_enum }

macro_rules! make_exif_struct {
    ($unused:tt, [ $( ($name:ident, $id:expr, $str:expr) ),* ]) => {

        impl ExifBlock {
            fn init_exif_tags(&mut self) {
                $( self.exif_tags.push( ExifTag{ id: $id, enu: ExifTagId:: $name , name: $str.to_string() }); )*
            }
        }
    };
}
apply_exif_tags!{ make_exif_struct }


/*macro_rules! make_exif_tag {
    ($val:expr, [ $( ($name:ident, $id:expr, $str:expr) ),* ]) => {
        match $val {
            $( $name => Some( ExifTag{ $val, $id, $str} ) )*
            _ => None
        }
    };
}

pub fn get_exif_tag(tag_id: u16) -> Option<ExifTag> {
    apply_exif_tags! { make_exif_tag, tag_id }
}*/


#[macro_export]
macro_rules! apply_gps_tags {
    ($callback:ident) => {
        $callback!((), [
            (VersionID,        0x00u16,"GPSVersionID"        ),
            (LatitudeRef,      0x01u16,"GPSLatitudeRef"      ), // 'N' = North | 'S' = South
            (Latitude,         0x02u16,"GPSLatitude"         ),
            (LongitudeRef,     0x03u16,"GPSLongitudeRef"     ), // 'E' = East | 'W' = West
            (Longitude,        0x04u16,"GPSLongitude"        ),
            (AltitudeRef,      0x05u16,"GPSAltitudeRef"      ), // 0 = Above Sea Level | 1 = Below Sea Level
            (Altitude,         0x06u16,"GPSAltitude"         ),
            (TimeStamp,        0x07u16,"GPSTimeStamp"        ),
            (Satelites,        0x08u16,"GPSSatelites"        ),
            (Status,           0x09u16,"GPSStatus"           ), //'A' = Measurement Active | 'V' = Measurement Void
            (MeasureMode,      0x0au16,"GPSMeasureMode"      ), // 2 = 2-Dimensional Measurement | 3 = 3-Dimensional Measurement
            (DOP,              0x0bu16,"GPSDOP"              ),
            (SpeedRef,         0x0cu16,"GPSSpeedRef"         ), // 'K' = km/h | 'M' = mph | 'N' = knots
            (Speed,            0x0du16,"GPSSpeed"            ),
            (TrackRef,         0x0eu16,"GPSTrackRef"         ), // 'M' = Magnetic North | 'T' = True North
            (Track,            0x0fu16,"GPSTrack"            ),
            (ImgDirectionRef,  0x10u16,"GPSImgDirectionRef"  ), // 'M' = Magnetic North | 'T' = True North
            (ImgDirection,     0x11u16,"GPSImgDirection"     ),
            (MapDatum,         0x12u16,"GPSMapDatum"         ),
            (DestLatitudeRef,  0x13u16,"GPSDestLatitudeRef"  ), // 'N' = North | 'S' = South
            (DestLatitude,     0x14u16,"GPSDestLatitude"     ),
            (DestLongitudeRef, 0x15u16,"GPSDestLongitudeRef" ), // 'E' = East | 'W' = West
            (DestLongitude,    0x16u16,"GPSDestLongitude"    ),
            (DestBearingRef,   0x17u16,"GPSDestBearingRef"   ), // 'M' = Magnetic North | 'T' = True North
            (DestBearing,      0x18u16,"GPSDestBearing"      ),
            (DestDistanceRef,  0x19u16,"GPSDestDistanceRef"  ), // 'K' = Kilometers | 'M' = Miles | 'N' = Nautical Miles
            (DestDistance,     0x1au16,"GPSDestDistance"     ),
            (ProcessingMethod, 0x1bu16,"GPSProcessingMethod" ), // "GPS", "CELLID", "WLAN" or "MANUAL"
            (AreaInformation,  0x1cu16,"GPSAreaInformation"  ),
            (DateStamp,        0x1du16,"GPSDateStamp"        ), // Format is YYYY:mm:dd
            (Differential,     0x1eu16,"GPSDifferential"     ), // 0 = No Correction | 1 = Differential Corrected
            (HPositioningError,0x1fu16,"GPSHPositioningError"),
            (UndefinedGpsTag,0xffffu16,"GPSUndefinedGpsTag"  )
        ]);
    };

    ($callback:ident, $extra:expr) => {
        $callback!($extra, [
            (VersionID,        0x00u16,"GPSVersionID"        ),
            (LatitudeRef,      0x01u16,"GPSLatitudeRef"      ), // 'N' = North | 'S' = South
            (Latitude,         0x02u16,"GPSLatitude"         ),
            (LongitudeRef,     0x03u16,"GPSLongitudeRef"     ), // 'E' = East | 'W' = West
            (Longitude,        0x04u16,"GPSLongitude"        ),
            (AltitudeRef,      0x05u16,"GPSAltitudeRef"      ), // 0 = Above Sea Level | 1 = Below Sea Level
            (Altitude,         0x06u16,"GPSAltitude"         ),
            (TimeStamp,        0x07u16,"GPSTimeStamp"        ),
            (Satelites,        0x08u16,"GPSSatelites"        ),
            (Status,           0x09u16,"GPSStatus"           ), //'A' = Measurement Active | 'V' = Measurement Void
            (MeasureMode,      0x0au16,"GPSMeasureMode"      ), // 2 = 2-Dimensional Measurement | 3 = 3-Dimensional Measurement
            (DOP,              0x0bu16,"GPSDOP"              ),
            (SpeedRef,         0x0cu16,"GPSSpeedRef"         ), // 'K' = km/h | 'M' = mph | 'N' = knots
            (Speed,            0x0du16,"GPSSpeed"            ),
            (TrackRef,         0x0eu16,"GPSTrackRef"         ), // 'M' = Magnetic North | 'T' = True North
            (Track,            0x0fu16,"GPSTrack"            ),
            (ImgDirectionRef,  0x10u16,"GPSImgDirectionRef"  ), // 'M' = Magnetic North | 'T' = True North
            (ImgDirection,     0x11u16,"GPSImgDirection"     ),
            (MapDatum,         0x12u16,"GPSMapDatum"         ),
            (DestLatitudeRef,  0x13u16,"GPSDestLatitudeRef"  ), // 'N' = North | 'S' = South
            (DestLatitude,     0x14u16,"GPSDestLatitude"     ),
            (DestLongitudeRef, 0x15u16,"GPSDestLongitudeRef" ), // 'E' = East | 'W' = West
            (DestLongitude,    0x16u16,"GPSDestLongitude"    ),
            (DestBearingRef,   0x17u16,"GPSDestBearingRef"   ), // 'M' = Magnetic North | 'T' = True North
            (DestBearing,      0x18u16,"GPSDestBearing"      ),
            (DestDistanceRef,  0x19u16,"GPSDestDistanceRef"  ), // 'K' = Kilometers | 'M' = Miles | 'N' = Nautical Miles
            (DestDistance,     0x1au16,"GPSDestDistance"     ),
            (ProcessingMethod, 0x1bu16,"GPSProcessingMethod" ), // "GPS", "CELLID", "WLAN" or "MANUAL"
            (AreaInformation,  0x1cu16,"GPSAreaInformation"  ),
            (DateStamp,        0x1du16,"GPSDateStamp"        ), // Format is YYYY:mm:dd
            (Differential,     0x1eu16,"GPSDifferential"     ), // 0 = No Correction | 1 = Differential Corrected
            (HPositioningError,0x1fu16,"GPSHPositioningError"),
            (UndefinedGpsTag,0xffffu16,"GPSUndefinedGpsTag"  )
        ])
    };
}

macro_rules! make_gps_enum {
    ($unused:tt, [ $( ($name:ident, $id:expr, $str:expr) ),* ]) => {
        #[derive(Serialize, Deserialize, Clone, Debug)]
        #[repr(u16)]
        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        pub enum GpsTagId {
            $( $name = $id ),*
        }
    };
}
apply_gps_tags!{ make_gps_enum }

macro_rules! make_gps_struct {
    ($unused:tt, [ $( ($name:ident, $id:expr, $str:expr) ),* ]) => {

        impl ExifBlock {
            fn init_gps_tags(&mut self) {
                $( self.gps_tags.push( GpsTag{ id: $id, enu: GpsTagId:: $name, name: $str.to_string() }); )*
            }
        }
    };
}
apply_gps_tags!{ make_gps_struct }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GpsTag {
    pub id: u16,
    pub enu: GpsTagId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExifTag {
    pub id: u16,
    pub enu: ExifTagId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialOrd, PartialEq)]
#[allow(non_camel_case_types)]
pub enum FMT {
   NONE,//- exif 3.0
   BYTE,
   STRING,
   USHORT,
   ULONG,
   URATIONAL,
   SBYTE, //- exif 3.0
   UNDEFINED,
   SSHORT, //- exif 3.0
   SLONG,
   SRATIONAL,
   SINGLE,//- exif 3.0
   DOUBLE,//- exif 3.0
   NUM_FORMATS,
   UTF_8 = 129, //+ exif 3.0
}
impl FMT {
    pub fn from(v: u16) -> Self {
        match v {
            0 => FMT::NONE,
            1 => FMT::BYTE,
            2 => FMT::STRING,
            3 => FMT::USHORT,
            4 => FMT::ULONG,
            5 => FMT::URATIONAL,
            6 => FMT::SBYTE,
            7 => FMT::UNDEFINED,
            8 => FMT::SSHORT,
            9 => FMT::SLONG,
           10 => FMT::SRATIONAL,
           11 => FMT::SINGLE,
           12 => FMT::DOUBLE,
          129 => FMT::UTF_8,
            _ => FMT::NUM_FORMATS,
        }
    }
    /*pub fn to(v:&str) -> Self {
        match v {
            "NONE"              => FMT::NONE,
            "BYTE"              => FMT::BYTE,
            "STRING"            => FMT::STRING,
            "USHORT"            => FMT::USHORT,
            "ULONG"             => FMT::ULONG,
            "URATIONAL"         => FMT::URATIONAL,
            "SBYTE"             => FMT::SBYTE,
            "UNDEFINED"         => FMT::UNDEFINED,
            "SSHORT"            => FMT::SSHORT,
            "SLONG"             => FMT::SLONG,
            "SRATIONAL"         => FMT::SRATIONAL,
            "SINGLE"            => FMT::SINGLE,
            "DOUBLE"            => FMT::DOUBLE,
            "UTF_8"             => FMT::UTF_8,
            _ => FMT::NUM_FORMATS,
        }
    }*/
}

const BYTESPERFORMAT: [usize; 13] = [0,1,1,2,4,8,1,1,2,4,8,4,8];

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct ExifBlock {
    pub exif_tags: Vec<ExifTag>,
    pub gps_tags: Vec<GpsTag>,
    pub raw_exif: Vec<u8>,
    pub raw_exif_length: usize,
    pub json_data: Option<Map<String, Value>>,
    pub entry_data_vector: Vec<ExifTagEntry>,
    pub lastexifrefd: usize,
    pub dirwiththumbnailptrs: usize,
    pub exifimagewidth: usize,
    pub motorola_order: bool,
    pub nesting_level: i32,
    pub make : String,
    pub thumbnailsize: usize,
    pub thumbnailoffset: usize,
}


impl Default for ExifBlock {
    fn default() -> Self {
        let mut tmp = Self {
            exif_tags: Vec::new(),
            gps_tags: Vec::new(),
            raw_exif: Vec::new(),
            raw_exif_length: 0,
            json_data: None,
            entry_data_vector: Vec::new(),
            lastexifrefd: 0,
            dirwiththumbnailptrs: 0,
            exifimagewidth: 0,
            motorola_order: false, //true: MM Big-endian, false: II Little-endian
            nesting_level: 0,
            make : "".into(),
            thumbnailsize: 0,
            thumbnailoffset: 0,
        };
        tmp.init_exif_tags();
        tmp.init_gps_tags();
        tmp
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExifTagEntry {
    pub name: String,
    pub value: serde_json::Value,
    pub offset: usize,
    //pub ifd: String,    // parent pl. "IFD0", "GPS" is unknown now
}

impl ExifBlock {
    
    pub fn find( &self, name: &str, occurrence: usize, case_sensitive: bool ) -> Option<&serde_json::Value> {
        self.entry_data_vector.iter()
            .filter(|entry| {
                if case_sensitive {
                    entry.name == name
                } else {
                    entry.name.eq_ignore_ascii_case(name)
                }
            })
            .nth(occurrence)
            .map(|entry| &entry.value)
    }

    pub fn find_tag( &self, name: &str, occurrence: usize, case_sensitive: bool ) -> Option<ExifTagEntry> {
        self.entry_data_vector.iter()
            .filter(|entry| {
                if case_sensitive {
                    entry.name == name
                } else {
                    entry.name.eq_ignore_ascii_case(name)
                }
            })
            .nth(occurrence).cloned()
    }

    pub fn fields(&self) -> impl Iterator<Item = (&String, &Value, &usize)> {
        self.entry_data_vector
            .iter()
            .map(|e| (&e.name, &e.value, &e.offset))
    }

    pub fn get_field(&self, fieldname: &str) -> Option<String> {
        if let Some(field) = self.find(fieldname,0,true) {
            let typ = field.get("type")?.as_str()?;
            let val = field.get("val")?;
            match typ {
                // Sztringek esetén az as_str() leveszi az idézőjeleket
                "STRING" | "ASCII" | "UNDEFINED" => {
                    //return Some(val.as_str().map(|s| s.to_string()));
                    return Some(val.to_string());
                },
                // Egész számok esetén
                "BYTE" | "SBYTE" | "USHORT" | "SSHORT" | "ULONG" | "SLONG" => {
                    return Some(val.to_string());
                },
                // Törtek (Rational) kiszámítása
                "URATIONAL" | "SRATIONAL" => {
                    let num = val.get(0)?.as_f64()?;
                    let den = val.get(1)?.as_f64()?;
                    if den == 0.0 { return None; }
                    return Some(format!("{:.2}", (num / den) as f32));
                },
                _ => return None,
            }
        }
        return None;
    }

    pub fn get_num_field(&self, fieldname: &str) -> Option<f32> {
        if let Some(field) = self.find(fieldname,0,true) {
            let typ = field.get("type")?.as_str()?;
            let val = field.get("val")?;
            match typ {
                // Egész számok esetén
                "BYTE" | "USHORT" | "ULONG" => {
                    return val.as_u64().map(|v| v as f32);
                },
                "SBYTE" | "SSHORT" | "SLONG" => {
                    return val.as_i64().map(|v| v as f32);
                },
                // Törtek (Rational) kiszámítása
                "URATIONAL" | "SRATIONAL" => {
                    if let Some(arr) = val.as_array() {
                        if arr.len() == 3 && arr[0].is_array() {
                            let mut deg = 0.0;
                            let divisors = [1.0, 60.0, 3600.0];

                            for (i, part) in arr.iter().enumerate() {
                                let n = part.get(0)?.as_f64()?;
                                let d = part.get(1)?.as_f64()?;
                                if d == 0.0 { return None; }
                                deg += (n / d) / divisors[i];
                            }
                            return Some(deg as f32);
                        }
                        let num = val.get(0)?.as_f64()?;
                        let den = val.get(1)?.as_f64()?;
                        if den == 0.0 { return None; }
                        return Some((num / den) as f32);
                    }
                },
                _ => return None,
            }
        }
        return None;
    }

    pub fn get_exif_tag(&self, id : u16) -> ExifTag {
        if let Some(tag) = self.exif_tags.iter().find(|t| t.id == id) {
            tag.clone()
        }
        else {
            ExifTag { id: id, enu: ExifTagId::UndefinedExifTag, name: format!("_{}",id) }
        }
    }
    
    pub fn get_gps_tag(&self, id : u16) -> GpsTag {
        if let Some(tag) = self.gps_tags.iter().find(|t| t.id == id) {
            tag.clone()
        }
        else {
            GpsTag { id: id, enu: GpsTagId::UndefinedGpsTag, name: format!("_{}",id) }
        }
    }
    
    fn read_buff_u16(&self, buff :&[u8], pos: usize) -> u16 {
        let bytes = buff[pos..pos + 2].try_into().unwrap();
        if self.motorola_order { u16::from_be_bytes(bytes) }
        else { u16::from_le_bytes(bytes) }
    }
    
    fn read_u16(&self, pos: usize) -> u16 {
        let bytes = self.raw_exif[pos..pos + 2].try_into().unwrap();
        if self.motorola_order { u16::from_be_bytes(bytes) }
        else { u16::from_le_bytes(bytes) }
    }
    
    fn read_u32(&self, pos: usize) -> u32 {
        let bytes = self.raw_exif[pos..pos + 4].try_into().unwrap();
        if self.motorola_order { u32::from_be_bytes(bytes) }
        else { u32::from_le_bytes(bytes) }
    }
    
    fn read_i32(&self, pos: usize) -> i32 {
        let bytes = self.raw_exif[pos..pos + 4].try_into().unwrap();
        if self.motorola_order { i32::from_be_bytes(bytes) }
        else { i32::from_le_bytes(bytes) }
    }
    
    fn read_f32(&self, pos: usize) -> f32 {
        let bytes = self.raw_exif[pos..pos + 4].try_into().unwrap();
        if self.motorola_order { f32::from_be_bytes(bytes) }
        else { f32::from_le_bytes(bytes) }
    }
    
    fn read_f64(&self, pos: usize) -> f64 {
        let bytes = self.raw_exif[pos..pos + 4].try_into().unwrap();
        if self.motorola_order { f64::from_be_bytes(bytes) }
        else { f64::from_le_bytes(bytes) }
    }

    fn convert_format_usize(&self, valueptr: usize, format:& FMT) -> usize {
       match format {
            FMT::BYTE   => (self.raw_exif[valueptr] as u8) as usize,
            FMT::SBYTE  => (self.raw_exif[valueptr] as i8) as usize,
            FMT::USHORT => (self.read_u16(valueptr)) as usize,
            FMT::SSHORT => (self.read_u16(valueptr) as i16) as usize,
            FMT::ULONG  => (self.read_u32(valueptr)) as usize,
            FMT::SLONG  => (self.read_i32(valueptr)) as usize,
            _ => (0) as usize, 
        }
    }

    pub fn patch_thumbnail(&mut self, new_thumb: &[u8]) {
        let offset = self.thumbnailoffset; // Ezt a open() során mentetted el
        let length = self.thumbnailsize;

        if length > 0 && self.raw_exif.len() >= offset + length && new_thumb.len() == length {
            self.raw_exif[offset..offset + length].copy_from_slice(new_thumb);
        }
    }

    pub fn patch_exifdata(&mut self, xres: f32, yres: f32, w: u32, h: u32) {
        if let Some(entry) = self.find_tag("XResolution",0,true) {
            let (nxf,nx) = if ((xres+0.5) as u32) as f32 == xres { (1.0,1u32) } else { (100000.0,100000u32) };
            let dx = (xres * nxf + 0.5) as u32;
            let mut bytes = if self.motorola_order { dx.to_be_bytes() } else { dx.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+4].copy_from_slice(&bytes);
            bytes = if self.motorola_order { nx.to_be_bytes() } else { nx.to_le_bytes() };
            self.raw_exif[entry.offset+4..entry.offset+8].copy_from_slice(&bytes);
        }

        if let Some(entry) = self.find_tag("YResolution",0,true) {
            let (nyf,ny) = if ((yres+0.5) as u32) as f32 == yres { (1.0,1u32) } else { (100000.0,100000u32) };
            let dy = (yres * nyf + 0.5) as u32;
            let mut bytes = if self.motorola_order { dy.to_be_bytes() } else { dy.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+4].copy_from_slice(&bytes);
            bytes = if self.motorola_order { ny.to_be_bytes() } else { ny.to_le_bytes() };
            self.raw_exif[entry.offset+4..entry.offset+8].copy_from_slice(&bytes);
        }

        if let Some(entry) = self.find_tag("Orientation",0,true) {
            let ori = 1u16;
            let bytes = if self.motorola_order { ori.to_be_bytes() } else { ori.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+2].copy_from_slice(&bytes);
        }

        if let Some(entry) = self.find_tag("DateTime",0,true) {
            let current_date = chrono::Local::now().format("%Y:%m:%d %H:%M:%S").to_string();
            let bytes = current_date.as_bytes();
            if bytes.len() <= 20 {
                self.raw_exif[entry.offset..entry.offset + bytes.len()].copy_from_slice(bytes);
            }
        }

        if let Some(entry) = self.find_tag("PixelXDimension",0,true) {
            let bytes = if self.motorola_order { w.to_be_bytes() } else { w.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+4].copy_from_slice(&bytes);
        }
        else if let Some(entry) = self.find_tag("ImageWidth",0,true) {
            let bytes = if self.motorola_order { w.to_be_bytes() } else { w.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+4].copy_from_slice(&bytes);
        }

        if let Some(entry) = self.find_tag("PixelYDimension",0,true) {
            let bytes = if self.motorola_order { h.to_be_bytes() } else { h.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+4].copy_from_slice(&bytes);
        }
        else if let Some(entry) = self.find_tag("ImageLength",0,true) {
            let bytes = if self.motorola_order { h.to_be_bytes() } else { h.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+4].copy_from_slice(&bytes);
        }

        if let Some(entry) = self.find_tag("Orientation",0,true) {
            let ori = 1u16;
            let bytes = if self.motorola_order { ori.to_be_bytes() } else { ori.to_le_bytes() };
            self.raw_exif[entry.offset..entry.offset+2].copy_from_slice(&bytes);
        }
    }

    pub fn open(&mut self, exifsection: &[u8],  length: usize) -> Result<ExifBlock, String> {
        let exifheader: [u8; 6] = [b'E',b'x',b'i',b'f',0,0];
        if exifsection[0..6] != exifheader {
            return Err("No exif header".into());
        }
        let motorola: [u8; 2] = [b'M',b'M'];
        let intel: [u8; 2] = [b'I',b'I'];
        if exifsection[6..8] == motorola {
            self.motorola_order = true;
        } else if exifsection[6..8] == intel {
            self.motorola_order = false;
        }else{
            return Err("Corrupt exif header: Invalid Exif alignment marker".into());
        }

        if self.read_buff_u16(&exifsection,8) != 0x2a {
            return Err("Corrupt exif header: Invalid Exif start (1)".into())
        }

        let firstoffset = self.read_buff_u16(&exifsection,10) as usize;
        if firstoffset < 8 || firstoffset > 32000 {
            return Err("Corrupt exif header: Suspicious offset of first IFD value".into());
        }

        self.raw_exif = exifsection.to_vec();
        self.raw_exif_length = length;
        self.lastexifrefd = 0;
        self.dirwiththumbnailptrs = 0;
        
        // First directory starts 16 bytes in.  All offset are relative to 8 bytes in.
        self.nesting_level+=1;
        let mut json = self.process_exif_dir(firstoffset+6, 6, length-6)?;
        self.nesting_level-=1;
        
        if self.thumbnailsize != 0 && self.thumbnailoffset != 0 {
            if self.thumbnailsize + self.thumbnailoffset <= length {
                let raw_bytes = &self.raw_exif[self.thumbnailoffset..self.thumbnailoffset + self.thumbnailsize];
                let value = json!(general_purpose::STANDARD.encode(raw_bytes));
                json.insert("Thumbnail".to_string(), value);
            }
        }
        let json_length = json!(length);
        json.insert("Exiflength".to_string(), json_length.clone());
        self.entry_data_vector.push( ExifTagEntry{ name:"Exiflength".to_string(), value:json_length, offset:0} );
        
        self.json_data = Some(json);
        Ok(self.clone())
    }


    fn dir_entry_addr(start: usize, entry: usize) -> usize {
        start + 2 + 12 * entry
    }


    fn process_exif_dir(&mut self, dirstart: usize, offsetbase: usize, exiflength: usize) ->  Result<Map<String, Value>, String> {

        let numdirentries = self.read_u16(dirstart) as usize;
        if self.nesting_level > 4 {
            return Err("Corrupt exif header: Maximum directory nesting exceeded".into());
        }

        let dirend = Self::dir_entry_addr(dirstart, numdirentries);
        if dirend+4 > offsetbase+exiflength {
            if dirend+2 == offsetbase+exiflength || dirend == offsetbase+exiflength {
                // version 1.3 of jhead would truncate a bit too much.
                // this also caught later on as well.
            }else{
                // note: files that had thumbnails trimmed with jhead 1.3 or earlier
                // might trigger this.
                return Err("Corrupt exif header: Illegally sized directory".into());
            }
        }
        if dirend > self.lastexifrefd { self.lastexifrefd = dirend; }

        let mut result = Map::new();

        for de_idx in 0..numdirentries {
            let idx = de_idx as usize;
            let direntry = Self::dir_entry_addr(dirstart, idx);
            
            let tag = self.get_exif_tag(self.read_u16(direntry));
            let format = FMT::from(self.read_u16(direntry+2));
            if format == FMT::NUM_FORMATS {
                return Err(format!("Corrupt exif header: Illegal number format {:?} for tag {:?}", format, tag.name));
            }
            let components = self.read_u32(direntry+4) as usize;
            let bytecount = components * BYTESPERFORMAT[format.clone() as usize];
            
            let mut json_tag: Map<String, Value> = Map::new();
            json_tag.insert("type".to_string(),json!(format));
            json_tag.insert("count".to_string(),json!(components));
            
            let valueptr = if bytecount > 4 {
                // if its bigger than 4 bytes, the dir entry contains an offset.
                let offsetval = self.read_u32(direntry+8) as usize;
                if offsetval+bytecount > exiflength {
                    return Err(format!("Corrupt exif header: Illegal value pointer for tag {:?}",tag.name));
                }
                offsetbase+offsetval
            }else{
                // 4 bytes or less and value is in the dir entry itself
                direntry+8
            };

            if self.lastexifrefd < valueptr+bytecount {
                // keep track of last byte in the exif header that was actually referenced.
                // that way, we know where the discardable thumbnail data begins.
                self.lastexifrefd = valueptr+bytecount;
            }

            match tag.enu {
                ExifTagId::GPSInfo => {
                        let subdirstart = offsetbase + self.read_u32(valueptr) as usize;
                        if subdirstart < offsetbase || subdirstart > offsetbase+exiflength {
                            return Err("Corrupt exif header: Illegal exif or interop ofset directory link".into());
                        }else{
                            self.nesting_level+=1;
                            let json = self.process_gps_info(subdirstart, offsetbase, exiflength)?;
                            self.nesting_level-=1;
                            result.insert(tag.name.clone(), json!(json));
                        }
                        continue;
                    },
                ExifTagId::EXIF_OFFSET | ExifTagId::INTEROP_OFFSET => {
                        let subdirstart = offsetbase + self.read_u32(valueptr) as usize;
                        if subdirstart < offsetbase || subdirstart > offsetbase+exiflength {
                            return Err("Corrupt exif header: Illegal exif or interop offset directory link".into());
                        }else{
                            self.nesting_level+=1;
                            let json = self.process_exif_dir(subdirstart, offsetbase, exiflength)?;
                            self.nesting_level-=1;
                            result.insert(tag.name.clone(), json!(json));
                        }
                        continue;
                    },
                ExifTagId::ThumbnailOffset => {
                        self.thumbnailoffset = self.convert_format_usize(valueptr, &format);
                        self.dirwiththumbnailptrs = dirstart;
                    },
                ExifTagId::ThumbnailLength => {
                        self.thumbnailsize = self.convert_format_usize(valueptr, &format);
                    },
                 _ => {},
                }

            let (value, insert_to_flat) = self.get_entry_value(format,valueptr,components,bytecount,
                    tag.enu == ExifTagId::Make,   tag.enu == ExifTagId::MakerNote && self.make == "Canon");

            let mut copy_json_tag = json_tag.clone();
            json_tag.insert("val".into(), value);
            let jsontag_value = serde_json::json!(json_tag);

            if insert_to_flat {
                self.entry_data_vector.push( ExifTagEntry{ name:tag.name.clone(), value:jsontag_value.clone(), offset:valueptr} );
            }
            else {
                let data = "long data";
                copy_json_tag.insert("val".into(), json!(data));
                let jsontag_value = serde_json::json!(copy_json_tag);
                self.entry_data_vector.push( ExifTagEntry{ name:tag.name.clone(), value:jsontag_value, offset:valueptr} );
            }
            result.insert(tag.name, jsontag_value);

        }

        // In addition to linking to subdirectories via exif tags,
        // there's also a potential link to another directory at the end of each
        // directory.  this has got to be the result of a comitee!
        if Self::dir_entry_addr(dirstart, numdirentries) + 4 <= offsetbase+exiflength {
             let offset = self.read_u32(dirstart+2+12*numdirentries) as usize;
             if offset != 0 {
                let subdirstart = offsetbase + offset;
                if subdirstart > offsetbase+exiflength {
                } else {
                   if subdirstart <= offsetbase+exiflength {
                      //inf->exiftext("%*ccontinued ",level*4,' ');
                      self.nesting_level+=1;
                      let json = self.process_exif_dir(subdirstart, offsetbase, exiflength)?;
                      self.nesting_level-=1;
                      result.insert("ExtraExifDir".to_string(), json!(json));
                   }
                }
             }
        } else {
             // The exif header ends before the last next directory pointer.
        }
        
        Ok(result)
    }


    //fn PrintFormatNumber(&mut self,valueptr: usize, format: FMT, bytecount: i32) {}
    fn process_gps_info(&mut self, dirstart: usize, offsetbase: usize, exiflength: usize) ->  Result<Map<String, Value>, String>  {
        let numdirentries = self.read_u16(dirstart) as usize;
        let dirend = Self::dir_entry_addr(dirstart, numdirentries);
        if dirend > (offsetbase+exiflength) {
            // Note: Files that had thumbnails trimmed with jhead 1.3 or earlier
            // might trigger this.
            return Err("Corrupt exif header: Illegally sized directory".into());
        }

        let mut result = Map::new();

        for de in 0..numdirentries {
            let idx = de as usize;
            let direntry = Self::dir_entry_addr(dirstart, idx);
            let tag = self.get_gps_tag(self.read_u16(direntry));
            let format = FMT::from(self.read_u16(direntry+2));
            if format == FMT::NUM_FORMATS {
                return Err(format!("Corrupt exif header: Illegal number format {:?} for tag {:?}", format, tag.name));
            }
            let components = self.read_u32(direntry+4) as usize;
            if components < 1 || components > 32768 {
                return Err(format!("Corrupt exif header: bad component number"));
            }
            let bytecount = components * BYTESPERFORMAT[format.clone() as usize];

            let mut json_tag: Map<String, Value> = Map::new();
            json_tag.insert("type".to_string(),json!(format));
            json_tag.insert("count".to_string(),json!(components));
            
            let valueptr = if bytecount > 4 {
                // if its bigger than 4 bytes, the dir entry contains an offset.
                let offsetval = self.read_u32(direntry+8) as usize;
                if offsetval+bytecount > exiflength {
                    return Err(format!("Corrupt exif header: Illegal value pointer for tag {:?}",tag.name));
                }
                offsetbase+offsetval
            }else{
                // 4 bytes or less and value is in the dir entry itself
                direntry+8
            };

            if self.lastexifrefd < valueptr+bytecount {
                // keep track of last byte in the exif header that was actually referenced.
                // that way, we know where the discardable thumbnail data begins.
                self.lastexifrefd = valueptr+bytecount;
            }

            let (value, insert_to_flat) = self.get_entry_value(format,valueptr,components,bytecount, false, false);

            let mut copy_json_tag = json_tag.clone();
            json_tag.insert("val".into(), value);
            let jsontag_value = serde_json::json!(json_tag);

            if insert_to_flat {
                self.entry_data_vector.push( ExifTagEntry{ name:tag.name.clone(), value:jsontag_value.clone(), offset:valueptr} );
            }
            else {
                let data = "long data";
                copy_json_tag.insert("val".into(), json!(data));
                let jsontag_value = serde_json::json!(copy_json_tag);
                self.entry_data_vector.push( ExifTagEntry{ name:tag.name.clone(), value:jsontag_value, offset:valueptr} );
            }
            result.insert(tag.name, jsontag_value);

        }
        
        Ok(result)
    }


    fn get_entry_value(&mut self, format: FMT, mut valueptr: usize, components: usize,
            bytecount: usize, is_make: bool, _is_note: bool) -> ( Value, bool) {
        match format {
            FMT::UNDEFINED | FMT::STRING | FMT::UTF_8 => {
                let raw_bytes = &self.raw_exif[valueptr..valueptr + bytecount];
                let clean_bytes = raw_bytes.split(|&b| b == 0).next().unwrap_or(&[]);
                let text = String::from_utf8_lossy(clean_bytes).to_string();
                if is_make {
                    self.make = text.clone();
                }
                return ( json!(text), true);
            },
            FMT::BYTE   => {
                //if is_note {
                //    json!(process_maker_note(valueptr, bytecount, offsetbase, exiflength))
                //}
                let raw_bytes = &self.raw_exif[valueptr..valueptr + bytecount];
                if bytecount<=120 { return (json!(raw_bytes),true); }
                else { return (json!(general_purpose::STANDARD.encode(raw_bytes)), false); }
            },
            FMT::SBYTE  =>
                if bytecount<=120 {
                    let signed_bytes: Vec<i8> = self.raw_exif[valueptr..valueptr + bytecount]
                        .iter().map(|&b| b as i8).collect();
                    return (json!(signed_bytes),true);
                }
                else {
                    let raw_bytes = &self.raw_exif[valueptr..valueptr + bytecount];
                    return (json!(general_purpose::STANDARD.encode(raw_bytes)), false);
                },
            FMT::USHORT => if components == 1 { return (json!(self.read_u16(valueptr)),true); },
            FMT::SSHORT => if components == 1 { return (json!(self.read_u16(valueptr) as i16),true); },
            FMT::ULONG  => if components == 1 { return (json!(self.read_u32(valueptr)),true); },
            FMT::SLONG  => if components == 1 { return (json!(self.read_i32(valueptr)),true); },
            FMT::URATIONAL | FMT::SRATIONAL => if components == 1 {
                let num = self.read_u32(valueptr);
                let den = self.read_u32(valueptr + 4);
                return (json!([num, den]),true);
            },
            FMT::SINGLE => if components == 1 { return (json!(self.read_f32(valueptr)),true); },
            FMT::DOUBLE => if components == 1 { return (json!(self.read_f64(valueptr)),true); },
            _ => return (json!(null),false), // Ismeretlen formátum esetén
        }
        
        match format {
            FMT::USHORT => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    values.push(self.read_u16(valueptr));
                    valueptr += 2;
                } 
                return (json!(values), true);
            },
            FMT::SSHORT => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    values.push(self.read_u16(valueptr) as i16);
                    valueptr+= 2;
                } 
                return (json!(values), true);
            },
            FMT::ULONG  => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    values.push(self.read_u32(valueptr));
                    valueptr += 4;
                } 
                return (json!(values), true);
            },
            FMT::SLONG  => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    values.push(self.read_i32(valueptr));
                    valueptr += 4;
                } 
                return (json!(values), true);
            },
            FMT::URATIONAL | FMT::SRATIONAL => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    let num = self.read_u32(valueptr);
                    let den = self.read_u32(valueptr + 4);
                    values.push([num, den]);
                    valueptr += 8;
                } 
                return (json!(values), true);
            },
            FMT::SINGLE => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    values.push(self.read_f32(valueptr));
                    valueptr += 4;
                } 
                return (json!(values), true);
            },
            FMT::DOUBLE => {
                let mut values = Vec::new();        
                for _i in 0..components {
                    values.push(self.read_f64(valueptr));
                    valueptr += 8;
                } 
                return (json!(values), true);
            },
            _ => return (json!(null),false),
        }
    }

    pub fn generate_fitted_thumbnail(&self, img: &image::RgbaImage) -> Vec<u8> {
        let  max_size = self.thumbnailsize;
        let thumb = image::DynamicImage::ImageRgba8(img.clone())
            .thumbnail(160, 120)
            .to_rgb8();

        let mut quality = 90;
        let mut result = Vec::new();

        // 2. Iteratív tömörítés
        loop {
            result.clear();
            let mut cursor = Cursor::new(&mut result);
            let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
            
            // Mentés próbája
            let _ = encoder.encode_image(&thumb);

            // Ha belefér vagy a minőség már nem csökkenthető tovább
            if result.len() <= max_size || quality <= 10 {
                break;
            }
            quality -= 10; // Lépésenként rontjuk a minőséget
        }
        if result.len() > max_size {
            return Vec::new();
        }

        // 3. Kitöltés (Padding)
        // Ha kisebb lett, nullákkal pótoljuk, hogy a hossza pontosan max_size legyen
        if result.len() < max_size {
            result.resize(max_size, 0x00);
        }

        result
    }

}



/*
JMESSAGE(JWRN_EXIF_1, "Corrupt exif header: Maximum directory nesting exceeded)")
JMESSAGE(JWRN_EXIF_2, "Corrupt exif header: Illegally sized directory")
JMESSAGE(JWRN_EXIF_3, "Corrupt exif header: Illegal number format %d for tag %04x")
JMESSAGE(JWRN_EXIF_4, "Corrupt exif header: Illegal value pointer for tag %04x")
JMESSAGE(JWRN_EXIF_5, "Corrupt exif header: More than %d date fields!  This is nuts")
JMESSAGE(JWRN_EXIF_6, "Corrupt exif header: Undefined rotation value %d")
JMESSAGE(JWRN_EXIF_7, "Corrupt exif header: Illegal exif or interop ofset directory link")
JMESSAGE(JWRN_EXIF_8, "Corrupt exif header: Illegal subdirectory link")
JMESSAGE(JWRN_EXIF_9, "Incorrect exif header")
JMESSAGE(JWRN_EXIF_10, "Corrupt exif header: Invalid Exif alignment marker")
JMESSAGE(JWRN_EXIF_11, "Corrupt exif header: Invalid Exif start (1)")
JMESSAGE(JWRN_EXIF_12, "Corrupt exif header: Suspicious offset of first IFD value")
*/