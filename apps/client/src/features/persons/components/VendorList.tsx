import React, { useState, useEffect } from "react";
import {
  deletePerson,
  getVendorPersons,
} from "@/features/persons/services/personService";
import { PersonDetailResponse } from "@/features/persons/types/person";
import DeleteConfirmationDialog from "@/shared/components/DeleteConfirmationDialog";
import { Button } from "@/shared/ui/button";
import { Plus, RefreshCw } from "lucide-react";
import PersonCard from "./PersonCard";
import PersonFormView from "./PersonFormView";

const VendorList: React.FC = () => {
  const [persons, setPersons] = useState<PersonDetailResponse[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState<boolean>(false);
  const [editingPersonId, setEditingPersonId] = useState<string | null>(null);
  const [viewingPersonId, setViewingPersonId] = useState<string | null>(null);
  const [formMode, setFormMode] = useState<"view" | "edit" | "create">(
    "create",
  );
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);
  const [personToDelete, setPersonToDelete] = useState<{
    id: string;
    name: string;
  } | null>(null);

  const fetchPersons = async (): Promise<void> => {
    try {
      setLoading(true);
      const data = await getVendorPersons();
      setPersons(data as PersonDetailResponse[]);
      setError(null);
    } catch (err) {
      setError("Failed to fetch vendors");
      console.error("Error fetching vendors:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchPersons();
  }, []);

  const handleViewPerson = (personId: string): void => {
    setViewingPersonId(personId);
    setEditingPersonId(null);
    setFormMode("view");
    setShowForm(true);
  };

  const handleEditPerson = (personId: string): void => {
    setEditingPersonId(personId);
    setViewingPersonId(null);
    setFormMode("edit");
    setShowForm(true);
  };

  const handleDeletePerson = (personId: string, personName: string): void => {
    setPersonToDelete({ id: personId, name: personName });
    setDeleteDialogOpen(true);
  };

  const handleAddPerson = (): void => {
    setEditingPersonId(null);
    setViewingPersonId(null);
    setFormMode("create");
    setShowForm(true);
  };

  const handleBackFromForm = (): void => {
    setShowForm(false);
    setEditingPersonId(null);
    setViewingPersonId(null);
    setFormMode("create");
  };

  const handleFormSave = (): void => {
    fetchPersons();
    handleBackFromForm();
  };

  const handlePersonDeleted = (): void => {
    fetchPersons();
    setPersonToDelete(null);
  };

  const handleCloseDeleteDialog = (): void => {
    setDeleteDialogOpen(false);
    setPersonToDelete(null);
  };

  if (showForm) {
    return (
      <PersonFormView
        personId={editingPersonId || viewingPersonId || undefined}
        personType="vendor"
        mode={formMode}
        onSave={handleFormSave}
        onBack={handleBackFromForm}
      />
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Vendors</h2>
        <div className="flex space-x-2">
          <Button
            variant="outline"
            onClick={fetchPersons}
            disabled={loading}
            size="sm"
          >
            <RefreshCw
              className={`h-4 w-4 mr-2 ${loading ? "animate-spin" : ""}`}
            />
            Refresh
          </Button>
          <Button onClick={handleAddPerson} size="sm">
            <Plus className="h-4 w-4 mr-2" />
            Add Vendor
          </Button>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
          {error}
        </div>
      )}

      {loading ? (
        <div className="flex justify-center items-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {persons.map((person) => (
            <PersonCard
              key={person.id}
              person={person}
              onView={handleViewPerson}
              onEdit={handleEditPerson}
              onDelete={(id) => handleDeletePerson(id, person.name)}
            />
          ))}
        </div>
      )}

      {!loading && persons.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          No vendors found. Click "Add Vendor" to create one.
        </div>
      )}

      <DeleteConfirmationDialog
        entityLabel="person"
        isOpen={deleteDialogOpen}
        onClose={handleCloseDeleteDialog}
        itemLabel={personToDelete?.name || personToDelete?.id}
        onConfirm={() => deletePerson(personToDelete?.id || "")}
        onDeleted={handlePersonDeleted}
      />
    </div>
  );
};

export default VendorList;
