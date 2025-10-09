"use client";

import { Check, Search } from "lucide-react";
import * as React from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

export type OptionType = {
	label: string;
	value: string;
	icon?: React.ComponentType<{ className?: string }>;
	color?: string;
};

interface SearchableMultiSelectProps {
	options: OptionType[];
	selected: string[];
	onChange: (selected: string[]) => void;
	className?: string;
	placeholder?: string;
	searchPlaceholder?: string;
}

export function SearchableMultiSelect({
	options,
	selected,
	onChange,
	className,
	placeholder = "Select options",
	searchPlaceholder = "Search...",
}: SearchableMultiSelectProps) {
	const [open, setOpen] = React.useState(false);
	const [searchValue, setSearchValue] = React.useState("");

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

	// Filter options based on search
	const filteredOptions = React.useMemo(() => {
		if (!searchValue) return options;
		const search = searchValue.toLowerCase();
		return options.filter((option) =>
			option.label.toLowerCase().includes(search),
		);
	}, [options, searchValue]);

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
			<PopoverContent className="w-[300px] p-0" align="start">
				<div className="flex items-center border-b px-3 pb-2 pt-3">
					<Search className="mr-2 h-4 w-4 shrink-0 opacity-50" />
					<Input
						placeholder={searchPlaceholder}
						value={searchValue}
						onChange={(e) => setSearchValue(e.target.value)}
						className="h-8 w-full border-0 bg-transparent p-0 placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-0"
					/>
				</div>
				<ScrollArea className="max-h-[300px]">
					<div className="p-1">
						{filteredOptions.length === 0 && (
							<div className="py-6 text-center text-sm text-muted-foreground">
								No options found
							</div>
						)}
						{filteredOptions.map((option) => {
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
									<Check
										className={cn(
											"h-4 w-4",
											isSelected ? "opacity-100" : "opacity-0",
										)}
									/>
									{option.color && (
										<span
											className="w-2 h-2 rounded-full"
											style={{ backgroundColor: option.color }}
										/>
									)}
									<span className="flex-1 text-left">{option.label}</span>
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
				</ScrollArea>
			</PopoverContent>
		</Popover>
	);
}
