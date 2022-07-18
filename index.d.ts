declare module "ardae-js" {
    class Engine {

        /** The internal data and state of the engine. Do not touch. */
        private data: unknown
        /** Prevents the object from being prematurely garbage collected. See `Engine.close()`. */
        private root: unknown

        /** Create and initialize new engine. */
        constructor()
 
        /**
         * Get all tracks currently on the mixer.
         */
        getTracks(): Track[]

        getTrack(key: number): Track

        /** 
         * Create new track, and add it to the mixer.
         * 
         * Can optionally take the `TrackData` returned by `Track.delete()` to reconstruct that track.
         */
        addTrack(data?: TrackData): Track

        /**
         * Create new array of tracks, and add them to the mixer.
         * 
         * Can either take a count, or an array of `TrackData`, 
         * which must all be unique, or the method will throw an `Error`.
         */
        addTracks(count: number): Track[]
        addTracks(data: TrackData[]): Track[]

        /**
         * Delete track, and remove it from the mixer. 
         * After this is done, calling any method on the track will throw an `Error`.
         * 
         * Returns data that can be passed to `Engine.addTrack/s()`, to reconstruct this track.
         */
        deleteTrack(track: Track): TrackData

        /**
         * Delete an array of tracks, and remove thme from the mixer. 
         * After this is done, calling any method on these tracks will throw an `Error`.
         * 
         * Returns an array of data that can be passed to `Engine.addTrack/s()`, to reconstruct these tracks.
         */
        deleteTracks(tracks: Track[]): TrackData[]

        /** 
         * Closes the engine down gracefully.
         * After this is called all other functions will throw an `Error`.
         */
        close(): void
    }

    interface Track {
        /** Unique identifier of the track. */
        readonly key: number

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

        /** 
         * Alias for `Engine.deleteTrack(this)`:
         * 
         * Delete track, and remove it from the mixer. 
         * After this is done, calling any method on the track will throw an `Error`.
         * 
         * Returns data that can be passed to `Engine.addTrack/s()`, to reconstruct this track.
         */
        delete(): TrackData
    }

    type TrackData = unknown

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
}