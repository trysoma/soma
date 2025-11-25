"use client";

import { format } from "date-fns";
import { Calendar as CalendarIcon, X } from "lucide-react";
import { useState } from "react";
import type { DateRange } from "react-day-picker";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";

interface DateRangePickerProps {
	value?: DateRange;
	onChange?: (range: DateRange | undefined) => void;
	placeholder?: string;
	className?: string;
}

export function DateRangePicker({
	value,
	onChange,
	placeholder = "Select date range",
	className,
}: DateRangePickerProps) {
	const [dateRange, setDateRange] = useState<DateRange | undefined>(value);

	const handleSelect = (range: DateRange | undefined) => {
		setDateRange(range);
		onChange?.(range);
	};

	const handleClear = () => {
		setDateRange(undefined);
		onChange?.(undefined);
	};

	return (
		<div className={cn("relative w-[280px]", className)}>
			<Popover>
				<PopoverTrigger asChild>
					<Button
						variant="outline"
						className={cn(
							"w-full justify-start text-left font-normal pr-10",
							!dateRange && "text-muted-foreground",
						)}
					>
						<CalendarIcon className="mr-2 h-4 w-4" />
						{dateRange?.from ? (
							dateRange.to ? (
								<>
									{format(dateRange.from, "MMM d, yyyy")} -{" "}
									{format(dateRange.to, "MMM d, yyyy")}
								</>
							) : (
								format(dateRange.from, "MMM d, yyyy")
							)
						) : (
							<span>{placeholder}</span>
						)}
					</Button>
				</PopoverTrigger>
				<PopoverContent className="w-auto p-0" align="start">
					<Calendar
						mode="range"
						defaultMonth={dateRange?.from}
						selected={dateRange}
						onSelect={handleSelect}
					/>
				</PopoverContent>
			</Popover>
			{dateRange && (
				<Button
					className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-sm opacity-70 hover:opacity-100 hover:bg-transparent dark:hover:bg-transparent"
					variant="ghost"
					onClick={handleClear}
					type="button"
				>
					<X className="h-4 w-4" />
				</Button>
			)}
		</div>
	);
}
