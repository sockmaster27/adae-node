declare module "ardae-js" {
    class Engine {

        /** The internal data and state of the engine. Do not touch. */
        private data: unknown
        /** Prevents the object from being prematurely garbage collected. See `Engine.close()`. */
        private root: unknown

        readonly tracks: Track[]

        /** Create and initialize new engine. */
        constructor()

        /** Create new track.
         * 
         * Can optionally take the `TrackData` returned by `Track.delete()` to reconstruct that track.
         */
        addTrack(data?: TrackData): Track

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
         * @param {number} value - Volume multiplier
         */
        setVolume(value: number): void

        /** Get current peak, long term peak and RMS (Root Mean Square) levels, for each channel. */
        readMeter(): { peak: [number, number], longPeak: [number, number], rms: [number, number] }

        /** Delete track.
         * 
         * Returns data that can be passed to `Engine.addTrack()`, to reconstruct this track.
         */
        delete(): TrackData
    }

    type TrackData = unknown
}