import React, { useState, useEffect } from "react";
import { deletePerson, getInternalPersons } from "@/features/persons/services/personService";
import { PersonDetailResponse } from "@/features/persons/types/person";
import DeleteConfirmationDialog from "@/shared/components/DeleteConfirmationDialog";
import { Button } from "@/shared/ui/button";
import { Plus, RefreshCw, User } from "lucide-react";
import PersonCard from "./PersonCard";
import PersonFormView from "./PersonFormView";

const InternalUserList: React.FC = () => {
  const [persons, setPersons] = useState<PersonDetailResponse[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState<boolean>(false);
  const [editingPersonId, setEditingPersonId] = useState<string | null>(null);
  const [viewingPersonId, setViewingPersonId] = useState<string | null>(null);
  const [formMode, setFormMode] = useState<"view" | "edit" | "create">("create");
  const [deleteDialogOpen, setDeleteDialogOpen] = useState<boolean>(false);
  const [personToDelete, setPersonToDelete] = useState<{ id: string; name: string } | null>(null);

  const fetchPersons = async (): Promise<void> => {
    try { setLoading(true); const data = await getInternalPersons(); setPersons(data as PersonDetailResponse[]); setError(null); }
    catch (err) { setError("Failed to fetch internal users"); console.error("Error fetching internal users:", err); }
    finally { setLoading(false); }
  };

  useEffect(() => { fetchPersons(); }, []);

  const handleViewPerson = (personId: string): void => { setViewingPersonId(personId); setEditingPersonId(null); setFormMode("view"); setShowForm(true); };
  const handleEditPerson = (personId: string): void => { setEditingPersonId(personId); setViewingPersonId(null); setFormMode("edit"); setShowForm(true); };
  const handleDeletePerson = (personId: string, personName: string): void => { setPersonToDelete({ id: personId, name: personName }); setDeleteDialogOpen(true); };
  const handleAddPerson = (): void => { setEditingPersonId(null); setViewingPersonId(null); setFormMode("create"); setShowForm(true); };
  const handleBackFromForm = (): void => { setShowForm(false); setEditingPersonId(null); setViewingPersonId(null); setFormMode("create"); };
  const handleFormSave = (): void => { fetchPersons(); handleBackFromForm(); };
  const handlePersonDeleted = (): void => { fetchPersons(); setPersonToDelete(null); };
  const handleCloseDeleteDialog = (): void => { setDeleteDialogOpen(false); setPersonToDelete(null); };

  if (showForm) {
    return <PersonFormView personId={editingPersonId || viewingPersonId || undefined} personType="internal" mode={formMode} onSave={handleFormSave} onBack={handleBackFromForm} />;
  }

  return (
    <div className="w-full">
      <div className="flex items-center justify-between mb-5">
        <h2 className="text-[17px] font-semibold text-foreground">Internal Users</h2>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={fetchPersons} disabled={loading} className="h-9 text-[13px]">
            <RefreshCw className={`h-3.5 w-3.5 mr-1.5 ${loading ? "animate-spin" : ""}`} /><span className="hidden sm:inline">Refresh</span>
          </Button>
          <Button size="sm" onClick={handleAddPerson} className="h-9 text-[13px]"><Plus className="h-3.5 w-3.5 mr-1.5" />Add User</Button>
        </div>
      </div>
      {error && <div className="text-[13px] text-destructive bg-destructive/8 rounded-xl px-4 py-3 mb-4">{error}</div>}
      {loading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (<div key={i} className="bg-card rounded-2xl border-[0.5px] border-border/60 p-4 space-y-3 animate-pulse"><div className="flex items-center gap-3"><div className="w-10 h-10 rounded-full skeleton" /><div className="space-y-1.5 flex-1"><div className="h-4 w-28 skeleton" /><div className="h-3 w-36 skeleton" /></div></div><div className="space-y-1.5"><div className="h-3 skeleton" /><div className="h-3 w-3/4 skeleton" /></div></div>))}
        </div>
      ) : persons.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <div className="w-12 h-12 rounded-2xl bg-secondary/60 flex items-center justify-center mb-4"><User className="w-6 h-6 text-muted-foreground/50" /></div>
          <h3 className="text-[17px] font-semibold text-foreground mb-1">No Users Yet</h3>
          <p className="text-[13px] text-muted-foreground mb-5 max-w-[240px]">Add your first internal user to get started.</p>
          <Button onClick={handleAddPerson}><Plus className="h-4 w-4 mr-2" />Add User</Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {persons.map((person) => (<PersonCard key={person.id} person={person} onView={handleViewPerson} onEdit={handleEditPerson} onDelete={(id) => handleDeletePerson(id, person.name)} />))}
        </div>
      )}
      <DeleteConfirmationDialog entityLabel="person" isOpen={deleteDialogOpen} onClose={handleCloseDeleteDialog} itemLabel={personToDelete?.name || personToDelete?.id} onConfirm={() => deletePerson(personToDelete?.id || "")} onDeleted={handlePersonDeleted} />
    </div>
  );
};

export default InternalUserList;
