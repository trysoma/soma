"use client";
import { useState } from "react";
import type { components } from "@/@types/openapi";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import $api from "@/lib/api-client.client";

type McpServerInstance =
	components["schemas"]["McpServerInstanceSerializedWithFunctions"];

interface CreateMcpServerDialogProps {
	isOpen: boolean;
	onClose: () => void;
	onSuccess: (instance: McpServerInstance) => void;
}

export function CreateMcpServerDialog({
	isOpen,
	onClose,
	onSuccess,
}: CreateMcpServerDialogProps) {
	const [id, setId] = useState("");
	const [name, setName] = useState("");
	const [error, setError] = useState<string | null>(null);

	const createMutation = $api.useMutation("post", "/api/bridge/v1/mcp-server");

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError(null);

		if (!id.trim()) {
			setError("ID is required");
			return;
		}

		if (!name.trim()) {
			setError("Name is required");
			return;
		}

		// Validate ID format (lowercase, alphanumeric, hyphens)
		if (!/^[a-z0-9-]+$/.test(id)) {
			setError("ID must contain only lowercase letters, numbers, and hyphens");
			return;
		}

		try {
			const result = await createMutation.mutateAsync({
				body: {
					id: id.trim(),
					name: name.trim(),
				},
			});
			setId("");
			setName("");
			onSuccess(result);
		} catch (err) {
			setError("Failed to create MCP server. Please try again.");
			console.error("Failed to create MCP server:", err);
		}
	};

	const handleClose = () => {
		setId("");
		setName("");
		setError(null);
		onClose();
	};

	return (
		<Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Create new MCP server</DialogTitle>
					<DialogDescription>
						Create a new MCP server instance to configure which functions are
						available to MCP clients.
					</DialogDescription>
				</DialogHeader>

				<form onSubmit={handleSubmit}>
					<div className="space-y-4 py-4">
						<div className="space-y-2">
							<Label htmlFor="id">ID</Label>
							<Input
								id="id"
								placeholder="my-mcp-server"
								value={id}
								onChange={(e) => setId(e.target.value)}
							/>
							<p className="text-xs text-muted-foreground">
								A unique identifier for this MCP server (lowercase, hyphens
								allowed)
							</p>
						</div>

						<div className="space-y-2">
							<Label htmlFor="name">Name</Label>
							<Input
								id="name"
								placeholder="My MCP Server"
								value={name}
								onChange={(e) => setName(e.target.value)}
							/>
							<p className="text-xs text-muted-foreground">
								A display name for this MCP server
							</p>
						</div>

						{error && <p className="text-sm text-destructive">{error}</p>}
					</div>

					<DialogFooter>
						<Button type="button" variant="outline" onClick={handleClose}>
							Cancel
						</Button>
						<Button type="submit" disabled={createMutation.isPending}>
							{createMutation.isPending ? "Creating..." : "Create"}
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	);
}
