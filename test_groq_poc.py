#!/usr/bin/env python3
"""
POC Groq API - Test de Performance pour rewrit
Objectif: Valider latence < 1s et qualité traduction
"""

import os
import time
from groq import Groq

# Configuration
API_KEY = os.getenv("GROQ_API_KEY", "VOTRE_KEY_ICI")
# Modèles disponibles 2026: llama-3.3-70b-versatile (recommandé), llama-3.1-8b-instant (ultra rapide)
MODEL = "llama-3.3-70b-versatile"  # Balance vitesse/qualité

# Test cases - Typiques pour rewrit
TEST_CASES = [
    # FR → EN
    ("FR→EN", "Bonjour le monde", "en"),
    ("FR→EN", "Comment allez-vous aujourd'hui?", "en"),
    ("FR→EN", "Je voudrais réserver une table pour deux personnes.", "en"),
    ("FR→EN", "L'intelligence artificielle transforme notre monde.", "en"),

    # EN → FR
    ("EN→FR", "Hello world", "fr"),
    ("EN→FR", "How are you today?", "fr"),
    ("EN→FR", "I would like to book a table for two people.", "fr"),
    ("EN→FR", "Artificial intelligence is transforming our world.", "fr"),

    # Cas complexes
    ("FR→EN", "C'est génial! 🎉 On a réussi.", "en"),
    ("EN→FR", "This is amazing! 🎉 We did it.", "fr"),
]

def test_groq_translation(client, direction, text, target_lang):
    """Test une traduction et mesure la latence"""

    # Prompt système
    system_prompt = f"Tu es un traducteur expert. Traduis UNIQUEMENT le texte en {target_lang}, sans explication."

    try:
        start = time.time()

        response = client.chat.completions.create(
            model=MODEL,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": text}
            ],
            temperature=0.3,  # Plus déterministe
            max_tokens=500
        )

        end = time.time()
        latency_ms = (end - start) * 1000

        translation = response.choices[0].message.content.strip()

        return {
            "success": True,
            "direction": direction,
            "input": text,
            "output": translation,
            "latency_ms": latency_ms,
            "tokens_input": response.usage.prompt_tokens if hasattr(response, 'usage') else 0,
            "tokens_output": response.usage.completion_tokens if hasattr(response, 'usage') else 0
        }

    except Exception as e:
        return {
            "success": False,
            "direction": direction,
            "input": text,
            "error": str(e),
            "latency_ms": 0
        }

def main():
    print("=" * 80)
    print("🚀 POC GROQ API - Test Performance pour rewrit")
    print("=" * 80)
    print()

    # Initialize client
    if API_KEY == "VOTRE_KEY_ICI":
        print("❌ ERREUR: Définissez votre GROQ_API_KEY!")
        print("   export GROQ_API_KEY='votre_clé_ici'")
        return

    client = Groq(api_key=API_KEY)
    print(f"✅ Client Groq initialisé")
    print(f"📋 Modèle: {MODEL}")
    print(f"🧪 Tests à exécuter: {len(TEST_CASES)}")
    print()

    # Run tests
    results = []
    print("-" * 80)

    for i, (direction, text, target_lang) in enumerate(TEST_CASES, 1):
        print(f"\n[Test {i}/{len(TEST_CASES)}] {direction}")
        print(f"  Input:  {text}")

        result = test_groq_translation(client, direction, text, target_lang)
        results.append(result)

        if result["success"]:
            print(f"  Output: {result['output']}")
            print(f"  ⚡ Latence: {result['latency_ms']:.0f}ms")

            # Indicateur visuel performance
            if result['latency_ms'] < 500:
                print(f"  ✅ EXCELLENT (< 500ms)")
            elif result['latency_ms'] < 1000:
                print(f"  ✅ BON (< 1s)")
            elif result['latency_ms'] < 2000:
                print(f"  ⚠️  ACCEPTABLE (< 2s)")
            else:
                print(f"  ❌ TROP LENT (> 2s)")
        else:
            print(f"  ❌ ERREUR: {result['error']}")

    # Statistics
    print("\n" + "=" * 80)
    print("📊 RÉSULTATS GLOBAUX")
    print("=" * 80)

    successes = [r for r in results if r["success"]]
    failures = [r for r in results if not r["success"]]

    if successes:
        latencies = [r["latency_ms"] for r in successes]

        print(f"\n✅ Succès: {len(successes)}/{len(results)} ({len(successes)/len(results)*100:.0f}%)")
        print(f"❌ Échecs: {len(failures)}/{len(results)}")
        print()
        print(f"⚡ Performance:")
        print(f"   Latence Min:  {min(latencies):.0f}ms")
        print(f"   Latence Max:  {max(latencies):.0f}ms")
        print(f"   Latence Moy:  {sum(latencies)/len(latencies):.0f}ms")
        print(f"   Latence P95:  {sorted(latencies)[int(len(latencies)*0.95)]:.0f}ms" if len(latencies) > 1 else "")

        # Tokens
        total_tokens_in = sum(r.get("tokens_input", 0) for r in successes)
        total_tokens_out = sum(r.get("tokens_output", 0) for r in successes)
        print()
        print(f"📊 Tokens:")
        print(f"   Input:  {total_tokens_in} tokens")
        print(f"   Output: {total_tokens_out} tokens")
        print(f"   Total:  {total_tokens_in + total_tokens_out} tokens")

    # Decision
    print("\n" + "=" * 80)
    print("🎯 DÉCISION")
    print("=" * 80)

    if successes:
        avg_latency = sum(r["latency_ms"] for r in successes) / len(successes)
        success_rate = len(successes) / len(results) * 100

        if avg_latency < 1000 and success_rate >= 90:
            print("\n✅ ✅ ✅ GO POUR INTEGRATION! ✅ ✅ ✅")
            print(f"   • Latence moyenne: {avg_latency:.0f}ms < 1000ms ✅")
            print(f"   • Taux succès: {success_rate:.0f}% >= 90% ✅")
            print()
            print("👉 Next Step: Intégrer Groq dans Tauri (remplacer Mistral 7b)")
        elif avg_latency < 2000 and success_rate >= 80:
            print("\n⚠️  RÉSULTATS MITIGÉS - À AMÉLIORER")
            print(f"   • Latence: {avg_latency:.0f}ms (viser < 1000ms)")
            print(f"   • Taux succès: {success_rate:.0f}% (viser >= 90%)")
            print()
            print("👉 Options:")
            print("   1. Tester autre modèle Groq (llama-3.3-70b-versatile)")
            print("   2. Tester Claude Haiku")
            print("   3. Optimiser prompts")
        else:
            print("\n❌ PERFORMANCE INSUFFISANTE")
            print(f"   • Latence: {avg_latency:.0f}ms > 2000ms ❌")
            print(f"   • Taux succès: {success_rate:.0f}% < 80% ❌")
            print()
            print("👉 Recommandation: Tester Claude Haiku à la place")
    else:
        print("\n❌ TOUS LES TESTS ONT ÉCHOUÉ")
        print("👉 Vérifiez votre API key et connexion internet")

    print("\n" + "=" * 80)

if __name__ == "__main__":
    main()
