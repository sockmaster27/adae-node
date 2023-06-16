// "#type" is used to disallow two types being mixed up.
// It doesn't actually exist at runtime.

declare module "adae-js" {
    abstract class ExposedObject {
        /** The internal data and state of the engine. Do not touch. */
        private data: unknown
        /** Prevents the object from being prematurely garbage collected. */
        private root?: unknown
    }

    class Engine extends ExposedObject {
        #type: "Engine"

        /** Create and initialize new engine. */
        constructor()

        /**
         * Get the master track, which is always present on the mixer.
         */
        getMaster(): MasterTrack

        /**
         * Get all tracks currently on the mixer.
         */
        getAudioTracks(): AudioTrack[]

        /** 
         * Create new audio track, and add it to the mixer.
         */
        addAudioTrack(): AudioTrack

        /**
         * Create new array of tracks, and add them to the mixer.
         */
        addAudioTracks(count: number): AudioTrack[]

        /**
         * Delete audio track, and remove it from the mixer. 
         * After this is done, calling any method on the track will throw an `Error`.
         * 
         * Returns a state that can be passed to `Engine.reconstructAudioTrack/s()`, to reconstruct this track.
         */
        deleteAudioTrack(audioTrack: AudioTrack): AudioTrackState

        /**
         * Delete an array of audio tracks, and remove thme from the mixer. 
         * After this is done, calling any method on these tracks will throw an `Error`.
         * 
         * Returns an array of data that can be passed to `Engine.reconstructAudioTrack/s()`, to reconstruct these tracks.
         */
        deleteAudioTracks(audioTracks: AudioTrack[]): AudioTrackState[]

        /**
         * Reconstruct an audio track that has been deleted.
         * 
         * The state can be obtained by calling `AudioTrack.delete()` or `Engine.deleteAudioTrack/s()`.
         */
        reconstructAudioTrack(state: AudioTrackState): AudioTrack
        /**
         * Reconstruct an array of audio tracks that have been deleted.
         * 
         * The states can be obtained by calling `AudioTrack.delete()` or `Engine.deleteAudioTrack/s()`.
         */
        reconstructAudioTracks(states: AudioTrackState[]): AudioTrack[]


        importAudioClip(path: string): AudioClip

        /** 
         * Closes down the engine gracefully.
         * After this is called all other methods will throw an `Error`.
         */
        close(): void
    }

    abstract class Track extends ExposedObject {
        getPanning(): number
        setPanning(value: number): void

        getVolume(): number
        /**
         * Sets the output volume of the track.
         */
        setVolume(value: number): void

        /** 
         * Get current peak, long term peak and RMS (Root Mean Square) levels, for each channel. 
         * Values are scaled and smoothed.
         */
        readMeter(): { peak: [number, number], longPeak: [number, number], rms: [number, number] }
        /** 
         * Cut off smoothing of RMS, and snap it to its current unsmoothed value.
         * 
         * Should be called before `readMeter()` is called the first time or after a long break,
         * to avoid meter sliding in place from zero or a very old value.
         */
        snapMeter(): void
    }

    class MasterTrack extends Track {
        #type: "MasterTrack"
        private constructor()
    }

    class AudioTrack extends Track {
        #type: "AudioTrack"
        private constructor()

        /** Unique identifier of the track. */
        key(): number


        addClip(clip: AudioClip, start: Timestamp, length?: Timestamp): void

        /** 
         * Alias for `Engine.deleteAudioTrack(this)`:
         * 
         * Delete track, and remove it from the mixer. 
         * After this is done, calling any method on the track will throw an `Error`.
         * 
         * Returns a state that can be passed to `Engine.reconstructAudioTrack/s()`, to reconstruct this track.
         */
        delete(): AudioTrackState
    }

    abstract class TrackState extends ExposedObject { }
    class MasterTrackState extends TrackState {
        #type: "MasterTrackState"
        private constructor()
    }
    class AudioTrackState extends TrackState {
        #type: "AudioTrackState"
        private constructor()
    }

    abstract class Clip extends ExposedObject {
        key(): number
    }
    class AudioClip extends Clip {
        #type: "AudioClip"
        private constructor()
    }

    class Timestamp extends ExposedObject {
        #type: "Timestamp"
        private constructor()

        static zero(): Timestamp
        static fromBeatUnits(beatUnits: number): Timestamp

        equals(other: Timestamp): boolean
        getBeatUnits(): number
    }

    /**
     * Scaling function used by Track.readMeter().
     * 
     * Read only.
     */
    function meterScale(value: number): number
    /**
     * Inverse of scaling used by Track.readMeter().
     * 
     * Useful for volume slider in proximity to a meter.
     */
    function inverseMeterScale(value: number): number

    /** 
     * Await next debug print.
     * 
     * If package is built in release mode (default), this will never resolve.
     */
    function getDebugOutput(): Promise<string>

    /**
     * Rejects if the engine has crashed.
     * 
     * Resolves when `stopListeningForCrash` is called. If this is never called, the process might hang.
     * 
     * Whenever possible, crashes will be thrown as exceptions by the function that caused them.
     * This function only exists to catch crashes that happen in the realtime thread, which cannot otherwise be caught by the JS engine.
     * 
     * If this reports a crash, this entire extension-module (including all open engines) will be in an undefined state, and should be closed.
     */
    function listenForCrash(): Promise<void>
    /**
     * Makes `listenForCrash` resolve.
     * 
     * This should be called when the engine is closed, to avoid hanging the process.
     */
    function stopListeningForCrash(): void
}
