declare module "ardae-js" {
    class Engine {

        /** The internal data and state of the engine. Do not touch. */
        private data: any
        /** Prevents the object from being prematurely garbage collected. See `Engine.close()`. */
        private root: any

        readonly tracks: Track[]

        /** Create and initialize new engine. */
        constructor()

        addTrack(): Track

        /**
         * Closes the engine down gracefully.
         * After this is called all other functions will throw an `Error`.
         */
        close(): void
    }

    interface Track {
        /** Unique identifier of the track. */
        readonly key: number

        setPanning(value: number): void

        /**
         * Sets the output volume of the track.
         * @param {number} value - Volume multiplier
         */
        setVolume(value: number): void

        /** Get current peak, long term peak and RMS (Root Mean Square) levels, for each channel. */
        readMeter(): { peak: [number, number], longPeak: [number, number], rms: [number, number] }


        delete(): void
    }
}