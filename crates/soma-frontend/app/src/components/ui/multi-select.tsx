"use client";

import { X } from "lucide-react";
import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";

export type OptionType = {
	label: string;
	value: string;
	icon?: React.ComponentType<{ className?: string }>;
	color?: string;
};

interface MultiSelectProps {
	options: OptionType[];
	selected: string[];
	onChange: (selected: string[]) => void;
	className?: string;
	placeholder?: string;
}

export function MultiSelect({
	options,
	selected,
	onChange,
	className,
	placeholder = "Select options",
}: MultiSelectProps) {
	const [open, setOpen] = React.useState(false);

	const handleSelect = (value: string) => {
		if (selected.includes(value)) {
			onChange(selected.filter((item) => item !== value));
		} else {
			onChange([...selected, value]);
		}
	};

	const selectedOptions = options.filter((option) =>
		selected.includes(option.value),
	);

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					variant="outline"
					className={cn(
						"w-full justify-start text-left font-normal",
						!selected.length && "text-muted-foreground",
						className,
					)}
				>
					{selected.length > 0 ? (
						<div className="flex gap-1 flex-wrap">
							{selected.length > 2 ? (
								<Badge variant="secondary" className="px-1 rounded-sm">
									{selected.length} selected
								</Badge>
							) : (
								selectedOptions.map((option) => (
									<Badge
										variant="secondary"
										key={option.value}
										className="px-1 rounded-sm"
									>
										{option.label}
									</Badge>
								))
							)}
						</div>
					) : (
						<span>{placeholder}</span>
					)}
				</Button>
			</PopoverTrigger>
			<PopoverContent className="w-full p-2" align="start">
				<div className="space-y-1">
					{options.map((option) => {
						const isSelected = selected.includes(option.value);
						return (
							<button
								key={option.value}
								onClick={() => handleSelect(option.value)}
								className={cn(
									"flex items-center gap-2 w-full px-2 py-1.5 text-sm rounded hover:bg-accent transition-colors",
									isSelected && "bg-accent",
								)}
								type="button"
							>
								{option.color && (
									<span
										className="w-2 h-2 rounded-full"
										style={{ backgroundColor: option.color }}
									/>
								)}
								<span className="flex-1 text-left">{option.label}</span>
								{isSelected && <X className="h-3 w-3" />}
							</button>
						);
					})}
					{selected.length > 0 && (
						<>
							<div className="border-t my-1" />
							<button
								onClick={() => onChange([])}
								className="flex items-center gap-2 w-full px-2 py-1.5 text-sm rounded hover:bg-accent transition-colors text-muted-foreground"
								type="button"
							>
								Clear all
							</button>
						</>
					)}
				</div>
			</PopoverContent>
		</Popover>
	);
}
