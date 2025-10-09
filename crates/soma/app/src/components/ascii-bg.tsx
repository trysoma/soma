"use client";
import React, { useState, useEffect, useRef } from "react";

const AsciiBg = () => {
	const [frames, setFrames] = useState<string[]>([]);
	const [currentFrame, setCurrentFrame] = useState(0);
	const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
	const [opacity, setOpacity] = useState(0.5);
	const containerRef = useRef<HTMLDivElement>(null);

	// Handle scroll-based opacity
	useEffect(() => {
		const handleScroll = () => {
			const scrollY = window.scrollY;
			const viewportHeight = window.innerHeight;

			// Calculate opacity: 0.8 at top, 0 at viewport height
			const newOpacity = Math.max(0, 0.5 - (scrollY / viewportHeight) * 0.5);
			setOpacity(newOpacity);
		};

		window.addEventListener("scroll", handleScroll);
		return () => window.removeEventListener("scroll", handleScroll);
	}, []);

	// Update dimensions on mount and resize
	// Load ASCII file from public folder
	useEffect(() => {
		const updateDimensions = () => {
			if (containerRef.current) {
				setDimensions({
					width: containerRef.current.offsetWidth,
					height: containerRef.current.offsetHeight,
				});
			}
		};

		updateDimensions();
		window.addEventListener("resize", updateDimensions);
		return () => window.removeEventListener("resize", updateDimensions);
	}, []);
	// Parse ASCII animation - frames separated by blank lines
	const parseASCIIAnimation = (content: string) => {
		const parsedFrames = content.split("\n\n").filter((frame) => frame.trim());

		if (parsedFrames.length > 1) {
			setFrames(parsedFrames);
			console.log(`Loaded ${parsedFrames.length} frames using blank lines`);
		} else {
			// Fallback to other delimiters
			const delimiters = ["\n---\n", "\n===\n", "\n###\n", "\n\n\n", "\f"];

			for (const delimiter of delimiters) {
				const testFrames = content
					.split(delimiter)
					.filter((frame) => frame.trim());
				if (testFrames.length > 1) {
					setFrames(testFrames);
					console.log(`Loaded ${testFrames.length} frames using delimiter`);
					return;
				}
			}

			// Single frame fallback
			setFrames([content]);
			console.log("Loaded 1 frame (no delimiters found)");
		}
	};

	// biome-ignore lint/correctness/useExhaustiveDependencies: <explanation>
	useEffect(() => {
		const loadASCIIFile = async () => {
			try {
				const response = await fetch("/ascii-animation.txt");
				const content = await response.text();
				parseASCIIAnimation(content);
			} catch (error) {
				console.error("Error loading ASCII file:", error);
			}
		};

		loadASCIIFile();
	}, []);

	
	// Animation loop
	useEffect(() => {
		if (frames.length > 1) {
			const interval = setInterval(() => {
				setCurrentFrame((prev) => (prev + 1) % frames.length);
			}, 100); // 10 FPS

			return () => clearInterval(interval);
		}
	}, [frames.length]);

	return (
		<div
			className="fixed inset-0 w-full h-full overflow-hidden pointer-events-none"
			style={{
				opacity: opacity,
			}}
		>
			<div
				ref={containerRef}
				className="fixed inset-0 w-full h-full overflow-hidden pointer-events-none"
				style={{
					opacity: 0.5,
				}}
			>
				<div
					className="w-full h-full font-mono text-xs leading-tight text-accent-foreground"
					style={{
						fontSize: "clamp(6px, 1vw, 12px)",
						lineHeight: "1.1",
					}}
				>
					{frames.length > 0 &&
						dimensions.width > 0 &&
						(() => {
							const currentFrameContent = frames[currentFrame];
							const frameLines = currentFrameContent.split("\n");
							const frameWidth = Math.max(
								...frameLines.map((line) => line.length),
							);
							const frameHeight = frameLines.length;

							// More accurate character sizing based on font
							const charWidth = 7; // Monospace character width approximation
							const lineHeightPx = 11; // Line height approximation

							const framePixelWidth = frameWidth * charWidth;
							const framePixelHeight = frameHeight * lineHeightPx;

							const tilesX = Math.ceil(dimensions.width / framePixelWidth) + 1;
							const tilesY =
								Math.ceil(dimensions.height / framePixelHeight) + 1;

							const tiles = [];
							for (let y = 0; y < tilesY; y++) {
								for (let x = 0; x < tilesX; x++) {
									tiles.push(
										<pre
											key={`${x}-${y}`}
											className="absolute whitespace-pre"
											style={{
												fontSize: "clamp(6px, 1vw, 12px)",
												lineHeight: "1.1",
												margin: 0,
												padding: 0,
												left: x * framePixelWidth,
												top: y * framePixelHeight,
											}}
										>
											{currentFrameContent}
										</pre>,
									);
								}
							}
							return tiles;
						})()}
				</div>
			</div>
		</div>
	);
};

export default AsciiBg;
