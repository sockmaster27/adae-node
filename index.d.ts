declare module "ardae-js" {
    class Engine {

        /**
         * The internal data and state of the engine. Do not touch.
         */
        private data: any

        /**
         * Create new engine.
         */
        constructor()

        /**
         * Sets the volume of the generated tone.
         * @param {number} value
         */
        setVolume(value: number): void

        /**
         * Get current peak level. This will be updated once every buffer.
         */
        getPeak(): number

        /**
         * Closes the engine down gracefully.
         * After this is called all other functions will throw an `Error`.
         */
        close(): void
    }
}