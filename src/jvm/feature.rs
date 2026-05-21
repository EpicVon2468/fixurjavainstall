use clap::ValueEnum;

// TODO: start-on-first-thread macOS (the LWJGL fix that breaks AWT/Swing)
#[non_exhaustive]
#[derive(Debug)]
#[derive_const(ValueEnum, Clone, PartialEq, Eq)]
pub enum Feature {
	/// Minimal JVM (JRE or no-Javadoc JDK).
	///
	/// If you don't know what this means & aren't a developer, you probably want this.
	Minimal,
	/// Dynamic Code Evolution Virtual Machine (enhanced runtime class redefinition) – <https://ssw.jku.at/dcevm/>.
	///
	/// Highly recommended for development, as it can allow for non-insignificant code changes without needing to restart the JVM.
	DCEVM,
	/// JDK Enhancement Proposal 519 (Compact Object Headers) – <https://openjdk.org/jeps/519>.
	///
	/// This feature relates to [Project Lilliput](https://openjdk.org/projects/lilliput/).
	///
	/// This feature can generally be considered stable, and is recommended for its strong performance benefits.
	///
	/// Additionally, some JVM vendors have backported this feature to previous versions, and [it may be enabled by default in future](https://openjdk.org/jeps/534).
	///
	/// See also:
	///
	/// - <https://openjdk.org/jeps/534>.
	/// - <https://openjdk.org/projects/lilliput/>.
	#[value(name = "jep-519", alias = "compact-object-headers")]
	JEP519,
	/// Wayland support (requires Vulkan) – <https://wiki.openjdk.org/spaces/wakefield/pages/77693134/Pure+Wayland+toolkit+prototype>.
	///
	/// Also known as [Project Wakefield](https://openjdk.org/projects/wakefield/).
	///
	/// See also:
	///
	/// - <https://openjdk.org/projects/wakefield/>.
	#[cfg(target_os = "linux")]
	#[value(aliases = vec!["wakefield", "wltoolkit", "wl"])]
	Wayland,
	/// OpenGL for AWT/Swing.
	///
	/// This has been bundled in OpenJDK for a long time, but isn't on by default.
	///
	/// macOS users, use Metal instead ([Apple has deprecated OpenGL on macOS](https://appleinsider.com/articles/18/06/28/why-macos-mojave-requires-metal----and-deprecates-opengl)).
	#[value(name = "opengl", alias = "gl")]
	OpenGL,
	/// Metal support for AWT/Swing (macOS) – <https://developer.apple.com/metal/>.
	///
	/// Also known as [Project Lanai](https://openjdk.org/projects/lanai/).
	///
	/// macOS users, use this instead of OpenGL ([Apple has deprecated OpenGL on macOS](https://appleinsider.com/articles/18/06/28/why-macos-mojave-requires-metal----and-deprecates-opengl/)).
	///
	/// See also:
	///
	/// - <https://openjdk.org/jeps/382>.
	/// - <https://openjdk.org/projects/lanai/>.
	#[cfg(target_os = "macos")]
	#[value(alias = "lanai")]
	Metal,
	/// Vulkan for AWT/Swing.
	///
	/// DEV NOTE: I'm not sure if this does anything when not used in conjunction with Wayland.
	#[value(alias = "vk")]
	Vulkan,
	/// Java Chromium Embedded Framework – <https://github.com/chromiumembedded/java-cef/>.
	///
	/// Webdev???  In my JVM???
	JCEF,
	/// Allows all Java modules to use the (soon to be) restricted native library access – <https://openjdk.org/jeps/472>.
	///
	/// JNI is to be replaced by [Project Panama](https://openjdk.org/projects/panama/) (Foreign Functions & Memory API).
	///
	/// Developers are discouraged from using JNI, and should instead favour the newer FFM API.
	///
	/// **This option will break the JVM if you are not on a version ≥ 24.**
	///
	/// See also:
	///
	/// - <https://inside.java/2024/12/09/quality-heads-up/>.
	/// - <https://docs.oracle.com/en/java/javase/26/core/restricted-methods.html>.
	/// - <https://openjdk.org/projects/panama/>.
	#[cfg(feature = "openjdk-restricted")]
	#[value(alias = "allow-native")]
	Native,
	/// Allows use of the (soon to be) restricted sun.misc.Unsafe API access – <https://openjdk.org/jeps/471>.
	///
	/// JNI is to be replaced by [Project Panama](https://openjdk.org/projects/panama/) (Foreign Functions & Memory API).
	///
	/// Developers are discouraged from using JNI, and should instead favour the newer FFM API.
	///
	/// **This option will break the JVM if you are not on a version ≥ 23.**
	///
	/// See also:
	///
	/// - <https://openjdk.org/projects/panama/>.
	#[cfg(feature = "openjdk-restricted")]
	#[value(alias = "allow-unsafe")]
	Unsafe,
	/// Allows final to *not* mean final – <https://openjdk.org/jeps/500>.
	///
	/// This is a terrible idea, but sometimes it's needed; Don't enable this unless you know what you're doing.
	///
	/// **This option will break the JVM if you are not on a version ≥ 26.**
	///
	/// See also:
	///
	/// - <https://www.youtube.com/watch?v=KoOrPzGC_7w>.
	#[cfg(feature = "openjdk-restricted")]
	#[value(alias = "no-final")]
	Mutate,
	/// Enables AWT/Swing font antialiasing.  This can improve readability and quality of text.
	FontFix,
	/// General fixes for NVIDIA GPUs on Linux.
	///
	/// Rendering may not work correctly or even at all without these.
	#[cfg(target_os = "linux")]
	NVIDIA,
	/// MUSL libc support – <https://musl.libc.org/>.
	///
	/// It is unlikely that a glibc JVM will work on MUSL.  Additionally, MUSL support is few and far between amongst JVM vendors.
	#[cfg(target_env = "musl")]
	MUSL,
	/// Bundles Kotlin with the JVM – <https://kotlinlang.org/>.
	Kotlin,
}
