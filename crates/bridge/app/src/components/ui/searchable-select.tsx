"use client";

import { Check, ChevronsUpDown } from "lucide-react";
import * as React from "react";
import { Button } from "@/components/ui/button";
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from "@/components/ui/command";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";

export interface SelectOption {
	value: string;
	label: string;
}

interface SearchableSelectProps {
	options: SelectOption[];
	value?: string;
	onChange?: (value: string) => void;
	onSearchChange?: (value: string) => void;
	placeholder?: string;
	searchPlaceholder?: string;
	emptyText?: string;
	className?: string;
	showAllOption?: boolean;
}

export function SearchableSelect({
	options,
	value,
	onChange,
	onSearchChange,
	placeholder = "Select...",
	searchPlaceholder = "Search...",
	emptyText = "No results found.",
	className,
	showAllOption = true,
}: SearchableSelectProps) {
	const [open, setOpen] = React.useState(false);
	const [searchValue, setSearchValue] = React.useState("");

	const handleSearchChange = (value: string) => {
		setSearchValue(value);
		onSearchChange?.(value);
	};

	// Add "All" option at the beginning if showAllOption is true
	const displayOptions = React.useMemo(() => {
		if (showAllOption && searchValue === "") {
			return [{ value: "", label: "All" }, ...options];
		}
		return options;
	}, [options, showAllOption, searchValue]);

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					variant="outline"
					role="combobox"
					aria-expanded={open}
					className={cn("w-[200px] justify-between", className)}
				>
					{value
						? options.find((option) => option.value === value)?.label || value
						: placeholder}
					<ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
				</Button>
			</PopoverTrigger>
			<PopoverContent className="w-[250px] p-0">
				<Command shouldFilter={false}>
					<CommandInput
						placeholder={searchPlaceholder}
						value={searchValue}
						onValueChange={handleSearchChange}
					/>
					<CommandList>
						{displayOptions.length === 0 && searchValue ? (
							<CommandEmpty>{emptyText}</CommandEmpty>
						) : null}
						<CommandGroup>
							{searchValue === "" && showAllOption && (
								<CommandItem
									value=""
									onSelect={() => {
										onChange?.("");
										setOpen(false);
										setSearchValue("");
									}}
								>
									<Check
										className={cn(
											"mr-2 h-4 w-4",
											value === "" ? "opacity-100" : "opacity-0",
										)}
									/>
									All
								</CommandItem>
							)}
							{displayOptions
								.filter((opt) => opt.value !== "")
								.map((option) => (
									<CommandItem
										key={option.value}
										value={option.value}
										onSelect={(currentValue) => {
											onChange?.(currentValue === value ? "" : currentValue);
											setOpen(false);
											setSearchValue("");
										}}
									>
										<Check
											className={cn(
												"mr-2 h-4 w-4",
												value === option.value ? "opacity-100" : "opacity-0",
											)}
										/>
										{option.label}
									</CommandItem>
								))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	);
}
