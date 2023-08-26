// "#type" is used to disallow two types being mixed up.
// It doesn't actually exist at runtime.

declare module "adae-node" {
    abstract class ExposedObject {
        /** The internal data and state of the engine. Do not touch. */
        private data: unknown;
        /** Prevents the object from being prematurely garbage collected. */
        private root?: unknown;
    }

    class Engine extends ExposedObject {
        #type: "Engine";

        /** Create a dummy engine, for testing purposes. */
        static dummy(): Engine;

        /**
         * Create and initialize new engine with the given configuration.
         *
         * If no configuration is given, the default configuration is used.
         */
        constructor(config?: config.Config);

        /** Restart the engine with the given config. */
        setConfig(config: config.Config): void;

        /**
         * Play timeline from the current playhead position.
         */
        play(): void;
        /**
         * Pause playback of the timeline, without resetting the playhead position.
         */
        pause(): void;
        /**
         * Set the current playhead position.
         *
         * This can be done both while the timeline is playing and while it is paused.
         */
        jumpTo(position: Timestamp): void;
        /**
         * Get the current playhead position.
         *
         * This reports the position as it currently is on the audio thread, which might have a slight delay in reacting to {@linkcode Engine.jumpTo()}.
         */
        getPlayheadPosition(): Timestamp;

        /**
         * Get the master track, which is always present on the mixer.
         */
        getMaster(): MasterTrack;

        /**
         * Get all tracks currently on the mixer.
         */
        getAudioTracks(): AudioTrack[];

        /**
         * Create new audio track, and add it to the mixer.
         */
        addAudioTrack(): AudioTrack;

        /**
         * Create new array of tracks, and add them to the mixer.
         */
        addAudioTracks(count: number): AudioTrack[];

        /**
         * Delete audio track, and remove it from the mixer.
         * After this is done, calling any method on the track will throw an {@linkcode Error}.
         *
         * Returns a state that can be passed to {@linkcode Engine.reconstructAudioTrack()}/{@linkcode reconstructAudioTracks()},
         * to reconstruct this track.
         */
        deleteAudioTrack(audioTrack: AudioTrack): AudioTrackState;

        /**
         * Delete an array of audio tracks, and remove thme from the mixer.
         * After this is done, calling any method on these tracks will throw an {@linkcode Error}.
         *
         * Returns an array of data that can be passed to {@linkcode Engine.reconstructAudioTrack()}/{@linkcode reconstructAudioTracks()},
         * to reconstruct these tracks.
         */
        deleteAudioTracks(audioTracks: AudioTrack[]): AudioTrackState[];

        /**
         * Reconstruct an audio track that has been deleted.
         *
         * The state can be obtained by calling {@linkcode AudioTrack.delete()} or {@linkcode Engine.deleteAudioTrack()}/{@linkcode deleteAudioTracks()}.
         */
        reconstructAudioTrack(state: AudioTrackState): AudioTrack;
        /**
         * Reconstruct an array of audio tracks that have been deleted.
         *
         * The states can be obtained by calling {@linkcode AudioTrack.delete()} or {@linkcode Engine.deleteAudioTrack()}/{@linkcode deleteAudioTracks()}.
         */
        reconstructAudioTracks(states: AudioTrackState[]): AudioTrack[];

        importAudioClip(path: string): StoredAudioClip;

        /**
         * Closes down the engine gracefully.
         * After this is called all other methods will throw an {@linkcode Error}.
         */
        close(): void;
    }

    abstract class Track extends ExposedObject {
        getPanning(): number;
        setPanning(value: number): void;

        getVolume(): number;
        /**
         * Sets the output volume of the track.
         */
        setVolume(value: number): void;

        /**
         * Get current peak, long term peak and RMS (Root Mean Square) levels, for each channel.
         * Values are scaled and smoothed.
         */
        readMeter(): {
            peak: [number, number];
            longPeak: [number, number];
            rms: [number, number];
        };
        /**
         * Cut off smoothing of RMS, and snap it to its current unsmoothed value.
         *
         * Should be called before {@linkcode Track.readMeter()} is called the first time or after a long break,
         * to avoid meter sliding in place from zero or a very old value.
         */
        snapMeter(): void;
    }

    class MasterTrack extends Track {
        #type: "MasterTrack";
        private constructor();
    }

    class AudioTrack extends Track {
        #type: "AudioTrack";
        private constructor();

        /** Unique identifier of the track. */
        key(): number;

        addClip(
            clip: StoredAudioClip,
            start: Timestamp,
            length?: Timestamp,
        ): AudioClip;

        /**
         * Alias for {@linkcode Engine.deleteAudioTrack()|Engine.deleteAudioTrack(this)}:
         *
         * Delete track, and remove it from the mixer.
         * After this is done, calling any method on the track will throw an {@linkcode Error}.
         *
         * Returns a state that can be passed to {@linkcode Engine.reconstructAudioTrack()}/{@linkcode Engine.reconstructAudioTracks()|reconstructAudioTracks()},
         * to reconstruct this track.
         */
        delete(): AudioTrackState;
    }

    abstract class TrackState extends ExposedObject {}
    class MasterTrackState extends TrackState {
        #type: "MasterTrackState";
        private constructor();
    }
    class AudioTrackState extends TrackState {
        #type: "AudioTrackState";
        private constructor();
    }

    /**
     * A clip that can be placed on the timeline.
     */
    abstract class StoredClip extends ExposedObject {
        key(): number;
    }
    class StoredAudioClip extends StoredClip {
        #type: "StoredAudioClip";
        private constructor();

        /**
         * Original sample rate of the audio file.
         */
        sampleRate(): number;

        /**
         * Get the full length of the clip in samples (per channel).
         *
         * This is relative to the sample rate of the clip, which is not necessarily the same as the sample rate of the engine
         * (See {@linkcode StoredAudioClip.sampleRate()}).
         */
        length(): number;
    }

    abstract class Clip extends ExposedObject {
        key(): number;

        start(): Timestamp;
        length(): Timestamp;
        end(): Timestamp;
    }
    class AudioClip extends Clip {
        #type: "AudioClip";
        private constructor();
    }

    class Timestamp extends ExposedObject {
        #type: "Timestamp";
        private constructor();

        /** Return the smallest of the two timestamps */
        static min(a: Timestamp, b: Timestamp): Timestamp;
        /** Return the largest of the two timestamps */
        static max(a: Timestamp, b: Timestamp): Timestamp;
        /** Check whether the two timestamps are equal to each other */
        static eq(a: Timestamp, b: Timestamp): boolean;

        /** Add `a` to `b` */
        static add(a: Timestamp, b: Timestamp): Timestamp;
        /**
         * Subtract `b` from `a`.
         * Throws {@linkcode Error} if result is less than zero.
         */
        static sub(a: Timestamp, b: Timestamp): Timestamp;
        /**
         * Multiplies timestamp `ts` with scalar `s`.
         * `s` will be truncated to an integer, and a
         * {@linkcode RangeError} will be thrown if `s` is less than zero.
         */
        static mul(ts: Timestamp, s: number): Timestamp;

        /**
         * The smallest possible timestamp representing the very beginning (regardless of unit)
         */
        static zero(): Timestamp;
        /**
         * The largest representable timestamp, convenient for comparisons.
         * Converting this to anything other than beat units may overflow.
         */
        static infinity(): Timestamp;

        /**
         * Create timestamp from beat units.
         * Parameter is truncated to an integer, and must be representable by a 32-bit unsigned integer.
         *
         * 1 beat = 1024 beat units
         */
        static fromBeatUnits(beatUnits: number): Timestamp;
        /**
         * Create timestamp from beats.
         * Parameter is truncated to an integer, and must be representable by a 32-bit unsigned integer.
         */
        static fromBeats(beats: number): Timestamp;
        /**
         * Create timestamp from samples.
         * `samples` parameter is truncated to an integer, and must be representable by a 64-bit unsigned integer.
         */
        static fromSamples(
            samples: number,
            sampleRate: number,
            bpm: number,
        ): Timestamp;

        /** 1 beat = 1024 beat units */
        getBeatUnits(): number;
        getBeats(): number;
        getSamples(sampleRate: number, bpm: number): number;
    }

    /**
     * Scaling function used by {@linkcode Track.readMeter()}.
     *
     * Read only.
     */
    function meterScale(value: number): number;
    /**
     * Inverse of scaling used by {@linkcode Track.readMeter()}.
     *
     * Useful for volume slider in proximity to a meter.
     */
    function inverseMeterScale(value: number): number;

    /**
     * Await next debug print.
     *
     * If package is built in release mode (default), this will never resolve.
     */
    function getDebugOutput(): Promise<string>;

    /**
     * Rejects if the engine has crashed.
     *
     * Resolves when {@linkcode stopListeningForCrash()} is called. If this is never called, the process might hang.
     *
     * Whenever possible, crashes will be thrown as exceptions by the function that caused them.
     * This function only exists to catch crashes that happen in the realtime thread, which cannot otherwise be caught by the JS engine.
     *
     * If this reports a crash, this entire extension-module (including all open engines) will be in an undefined state, and should be closed.
     */
    function listenForCrash(): Promise<void>;
    /**
     * Makes {@linkcode listenForCrash()} resolve.
     *
     * This should be called when the engine is closed, to avoid hanging the process.
     */
    function stopListeningForCrash(): void;

    namespace config {
        /**
         * Configuration of the engine.
         */
        class Config extends ExposedObject {
            #type: "Config";

            /**
             * Get a reasonable default configuration.
             */
            static default(): Config;

            constructor(outputDevice: OutputDevice, outputConfig: OutputConfig);
        }

        /**
         * Configuration of the output stream of the engine.
         *
         * All values should be chosen from a {@linkcode OutputConfigRange} reported by the specific {@linkcode OutputDevice},
         * through either {@linkcode OutputDevice.supportedConfigRanges()} or {@linkcode OutputDevice.defaultConfigRange()}.
         */
        interface OutputConfig {
            /**
             * The number of channels to output, i.e. 1 = mono and 2 = stereo.
             *
             * This setting has no impact on the number of channels used internally.
             * This is converted right before the samples are sent to the output device.
             */
            channels: number;
            /**
             * The format of the output samples.
             *
             * This setting has no impact on the format used internally, which is a mix of 32- and 64-bit floating point.
             * This is converted right before the samples are sent to the output device.
             */
            sampleFormat: SampleFormat;
            /**
             * The sample rate in Hz.
             *
             * This is used both internally and for the output device.
             */
            sampleRate: number;
            /**
             * The size of the internal buffers in samples.
             * If `null`, the default is used.
             */
            bufferSize: number | null;
        }

        /**
         * A range of valid values for a specific output device.
         *
         * This can be used to construct a valid {@linkcode OutputConfig} for a specific {@linkcode OutputDevice}.
         */
        class OutputConfigRange extends ExposedObject {
            #type: "OutputConfigRange";
            private constructor();

            channels(): number;
            sampleFormat(): SampleFormat;
            sampleRate(): { min: number; max: number };
            bufferSize(): { min: number; max: number } | null;

            /**
             * Get a reasonable default {@linkcode OutputConfig} for this range.
             */
            defaultConfig(): OutputConfig;
        }

        enum SampleFormat {
            Int8 = "i8",
            Int16 = "i16",
            Int32 = "i32",
            Int64 = "i64",
            IntUnsigned8 = "u8",
            IntUnsigned16 = "u16",
            IntUnsigned32 = "u32",
            IntUnsigned64 = "u64",
            Float32 = "f32",
            Float64 = "f64",
        }

        /**
         * A host is a specific audio backend, e.g. CoreAudio on macOS or WASAPI on Windows.
         *
         * A host can have zero, one or multiple {@linkcode OutputDevice}s, which are the actual audio devices that can be used.
         */
        class Host extends ExposedObject {
            #type: "Host";
            private constructor();

            /**
             * Get an array of all available hosts.
             */
            static available(): Host[];
            /**
             * Get the default host.
             */
            static default(): Host;

            /**
             * Get the name of the host.
             */
            name(): string;
            /**
             * Get an array of all available output devices for this host.
             * Might be empty.
             */
            outputDevices(): OutputDevice[];
            /**
             * Get the default output device for this host.
             */
            defaultOutputDevice(): OutputDevice | null;
        }

        /**
         * An output device is a specific audio device that can be used for output.
         *
         * It can be retrieved from a {@linkcode Host} through either {@linkcode Host.outputDevices()} or {@linkcode Host.defaultOutputDevice()}.
         */
        class OutputDevice extends ExposedObject {
            #type: "OutputDevice";
            private constructor();

            /**
             * Get the host that this output device belongs to.
             */
            host(): Host;
            /**
             * Get the name of the output device.
             */
            name(): string;
            /**
             * Get an array of all supported {@linkcode OutputConfigRange}s for this output device.
             */
            supportedConfigRanges(): OutputConfigRange[];
            /**
             * Get the default {@linkcode OutputConfigRange} for this output device.
             */
            defaultConfigRange(): OutputConfigRange;
        }
    }
}
