declare module "ardae-js" {
    class Engine {

        /** The internal data and state of the engine. Do not touch. */
        private data: unknown
        /** Prevents the object from being prematurely garbage collected. See `Engine.close()`. */
        private root: unknown

        readonly tracks: Track[]

        /** Create and initialize new engine. */
        constructor()

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
         * After this is done, calling any method on the track will throw an `Error`.
         * 
         * Returns an array of data that can be passed to `Engine.addTrack/s()`, to reconstruct these tracks.
         */
        deleteTracks(tracks: Track[]): TrackData[]

        getTrack(key: number): Track

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

        /** Get current peak, long term peak and RMS (Root Mean Square) levels, for each channel. */
        readMeter(): { peak: [number, number], longPeak: [number, number], rms: [number, number] }

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
     * Await next debug print.
     * 
     * If package is built in release mode (default), this will never resolve.
     */
    function getDebugOutput(): Promise<string>
}